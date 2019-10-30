# bz2github

## Getting an API key ##

See here: https://github.blog/2013-05-16-personal-api-tokens/

## Example usage ##

#### All bugs blocking another bug ###

```cargo run -- -p -a[API-KEY] -qquery_format=advanced -qf1=blocked -qo1=equals -qv1=1525312  -q"resolution=---" -qlimit=100 -rFirefoxGraphics/wr-planning -l"OS: Android"```
