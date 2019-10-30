# bz2github

## About ##

This is a simple tool to sync bugs from bugzilla.mozilla.org to github.
It will create or update corresponding issues into the github issues list of the specified repo or project.

## Getting an API key ##

See here: https://github.blog/2013-05-16-personal-api-tokens/

## Usage ##

```
Tool to sync issues from Bugzilla to Github

USAGE:
    bz2github.exe [FLAGS] [OPTIONS] --bugzilla_queryparam <bugzilla_queryparam>... --github_apikey <github_apikey> --github_repopath <github_repopath>

FLAGS:
    -h, --help       Prints help information
    -p, --preview    Only preview the operations without committing to github
    -V, --version    Prints version information

OPTIONS:
    -q, --bugzilla_queryparam <bugzilla_queryparam>...
            Bugzilla query parameter to sync issues from (multiple allowed). ex: keywords=topcrash and/or component=DOM"

    -a, --github_apikey <github_apikey>
            Personal API key to access github. (https://github.com/settings/tokens)

    -l, --github_labels <github_labels>...                Labels to assign to bugs (multiple allowed). ex: android"
    -r, --github_repopath <github_repopath>
            Path to the github repo where issues need to be created. ex: "orgs/FirefoxGraphics"
```

## Examples ##

#### All bugs blocking another bug ###

```cargo run -- -p -a[API-KEY] -qquery_format=advanced -qf1=blocked -qo1=equals -qv1=1525312  -q"resolution=---" -qlimit=100 -rFirefoxGraphics/wr-planning -l"OS: Android"```
