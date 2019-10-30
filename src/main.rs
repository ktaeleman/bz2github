extern crate restson;
#[macro_use]
extern crate serde_derive;
extern crate clap;
extern crate serde_json;

use clap::{Arg, ArgMatches, App};
//use std::env;
use restson::{Error, RestClient, RestPath};

pub fn encode(data: &str) -> String {
    let mut escaped = String::new();
    for b in data.as_bytes().iter() {
        match *b as char {
            // Accepted characters
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => escaped.push(*b as char),

            // Everything else is percent-encoded
            b => escaped.push_str(format!("%{:02X}", b as u32).as_str()),
        };
    }
    return escaped;
}

#[derive(Deserialize)]
struct HttpBugzillaBug {
    summary: String,
    id: u32
}

#[derive(Deserialize)]
struct HttpBugzillaBugs {
    bugs: Vec<HttpBugzillaBug>,
}

// impl RestPath<u32> for HttpBugzillaBugs {
//     fn get_path(issue_id: u32) -> Result<String, Error> {
//         Ok(format!("rest/bug/{}", issue_id))
//     }
// }

impl RestPath<()> for HttpBugzillaBugs {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("rest/bug"))
    }
}

// fn get_bugzilla_bug(issue_id: u32) -> HttpBugzillaBugs {
//     let mut client = RestClient::new("https://bugzilla.mozilla.org").unwrap();
//     client.get(issue_id).unwrap()
// }

#[derive(Serialize, Deserialize, Debug)]
struct HttpGithubIssue {
    #[serde(default)]
    number: u32,
    title: String,
    body: String,
    // Labels come back as label structs, but need to be uploaded as string list
    // We could just create a different structure for updating or creating issues
    #[serde(skip_deserializing)]
    labels: Vec<String>
}

impl RestPath<&String> for HttpGithubIssue {
    fn get_path(repopath: &String) -> Result<String, Error> {
        Ok(format!("repos/{}/issues", repopath))
    }
}

impl RestPath<(&String, u32)> for HttpGithubIssue {
    fn get_path(vars : (&String, u32)) -> Result<String, Error> {
        Ok(format!("repos/{}/issues/{}", vars.0, vars.1))
    }
}

#[derive(Deserialize)]
struct HttpGithubIssues(Vec<HttpGithubIssue>);

impl RestPath<&String> for HttpGithubIssues {
    fn get_path(repopath: &String) -> Result<String, Error> {
        Ok(format!("repos/{}/issues", repopath))
    }
}

struct Bz2GhClient {
    repopath : String,
    is_preview_only : bool,
    queryparams : Vec<(String, String)>,
    labels: Vec<String>,
    bz_client: RestClient,
    gh_client: RestClient,
}

impl Bz2GhClient {
    fn new(args: &ArgMatches) -> Bz2GhClient {
        let bz_client = RestClient::new("https://bugzilla.mozilla.org").unwrap();
        let apikey = args.value_of("github_apikey").unwrap();
        let repopath = args.value_of("github_repopath").unwrap().to_string();
        let is_preview_only = args.is_present("preview");
        let query = args.values_of("bugzilla_queryparam").unwrap();
        let labels : Vec<String> = match args.values_of("github_labels") {
            Some(v) => v.map(|s| s.to_string()).collect(),
            None => Vec::new(),
        };

        let mut queryparams = Vec::new();
        for val in query {
            let v: Vec<&str> = val.split("=").collect();
            if v.len() == 2 {
                let pair = (v[0].to_string(), v[1].to_string());
                queryparams.push(pair);
            }
        }

        let mut gh_client = RestClient::new("https://api.github.com").unwrap();
        gh_client.set_header("Authorization", &format!("token {}", apikey)[..]).unwrap();

        return Bz2GhClient {
            repopath,
            is_preview_only,
            queryparams,
            labels,
            bz_client,
            gh_client,
        }
    }

    fn get_bugzilla_bugs(&mut self) -> HttpBugzillaBugs {
        let param_data: Vec<(&str,&str)> = self.queryparams.iter().map(|(s1,s2)| (s1.as_ref(), s2.as_ref())).collect();
        println!("Running query: {:?}", param_data);
        self.bz_client.get_with((), &param_data[..]).unwrap()
    }

    fn find_issue_from_bug<'b>(issues: &'b HttpGithubIssues, bug : &HttpBugzillaBug)
    -> Option<&'b HttpGithubIssue>
    {
        for issue in &issues.0 {
            if issue.title.contains(&format!("[{}]", bug.id)) {
                return Some(&issue);
            }
        }
        return None;
    }

    fn update_issue(&mut self, issue: &HttpGithubIssue, bug: &HttpBugzillaBug)
    {
        let issue_data = HttpGithubIssue {
            number: issue.number,
            title: format!("{} [{}]", bug.summary, bug.id),
            body: format!("https://bugzilla.mozilla.org/show_bug.cgi?id={}", bug.id),
            labels: self.labels.to_vec(),
        };

        println!("Updating: {:?}", issue_data);
        if !self.is_preview_only {
            self.gh_client.patch((&self.repopath, issue.number), &issue_data).unwrap();
        }
    }

    fn create_issue(&mut self, bug: &HttpBugzillaBug)
    {
        let issue_data = HttpGithubIssue {
            number: 0,
            title: format!("{} [{}]", bug.summary, bug.id),
            body: format!("https://bugzilla.mozilla.org/show_bug.cgi?id={}", bug.id),
            labels: self.labels.to_vec(),
        };

        println!("Creating: {:?}", issue_data);
        if !self.is_preview_only {
            self.gh_client.post(&self.repopath, &issue_data).unwrap();
        }
    }

    fn sync_issues(&mut self, bugs: &HttpBugzillaBugs) {
        // Get Github issues so we can see whether to update the issue or create a new one
        let issues : HttpGithubIssues = self.gh_client.get(&self.repopath).unwrap();


        for bug in &bugs.bugs {
            println!("Processing bug {}: {}", bug.id, bug.summary);
            let existing_issue = Bz2GhClient::find_issue_from_bug(&issues, &bug);
            match existing_issue {
                Some(a) => self.update_issue(&a, &bug),
                None => self.create_issue(&bug),
            }
        }
    }
}

fn main() {
    //let apikey = env::var("GITHUB_API_KEY").unwrap();
    let args = App::new("Bugzilla to Github")
                    .version("0.1")
                    .author("Kris Taeleman <ktaeleman@mozilla.com>")
                    .about("Tool to sync issues from Bugzilla to Github")
                    .arg(Arg::with_name("github_apikey")
                        .short("a")
                        .long("github_apikey")
                        .help("Personal API key to access github. (https://github.com/settings/tokens)")
                        .takes_value(true)
                        .required(true))
                    .arg(Arg::with_name("bugzilla_queryparam")
                        .short("q")
                        .long("bugzilla_queryparam")
                        .multiple(true)
                        .help("Bugzilla query parameter to sync issues from (multiple allowed). ex: keywords=topcrash and/or component=DOM\"")
                        .takes_value(true)
                        .required(true))
                    .arg(Arg::with_name("preview")
                        .short("p")
                        .long("preview")
                        .help("Only preview the operations without committing to github"))
                    .arg(Arg::with_name("github_repopath")
                        .short("r")
                        .long("github_repopath")
                        .help("Path to the github repo where issues need to be created. ex: \"orgs/FirefoxGraphics\"")
                        .takes_value(true)
                        .required(true))
                    .arg(Arg::with_name("github_labels")
                        .short("l")
                        .long("github_labels")
                        .multiple(true)
                        .help("Labels to assign to bugs (multiple allowed). ex: android\"")
                        .takes_value(true))
                    .get_matches();

    let mut client = Bz2GhClient::new(&args);
    let data = client.get_bugzilla_bugs();
    if !data.bugs.is_empty() {
        client.sync_issues(&data);
    } else {
        println!("Zarro boogs found.");
    }


    println!("Done.");
}
