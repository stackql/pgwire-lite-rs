# pgwire-lite-rs

## Testing

## Robot testing

Per [`.github/workflows/regression.yml`](/.github/workflows/regression.yml).

### vscode debug

- Config in `launch.json` is dependent on [the `CodeLLDB` extension](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb).
- Also installed [the `rust-analyzer` extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

### Manual Dream Run

Build with `cargo build --release --bin client_test_harness`.

Then, presuming you have an appropriate `stackql` server running, the output of running `target/release/client_test_harness "SELECT repo, count(*) as has_starred FROM github.activity.repo_stargazers WHERE owner = 'stackql' and repo in ('stackql', 'stackql-deploy') and login = 'generalkroll0' GROUP BY repo;" "host=localhost port=5444"`, once github reaches rate limit:

```log
Query did some non-notify thing.
--- Notice 1 ---
sqlstate: 01000
detail: http response status code: 403, response body: {"message":"API rate limit exceeded for 110.144.44.79. (But here's the good news: Authenticated requests get a higher rate limit. Check out the documentation for more details.) If you reach out to GitHub Support for help, please include the request ID E19A:36FB4A:0813:0A62:67F1D9DD and timestamp 2025-04-06 01:33:17 UTC.","documentation_url":"https://docs.github.com/rest/overview/rate-limits-for-the-rest-api","status":"403"}
http response status code: 403, response body: {"message":"API rate limit exceeded for 110.144.44.79. (But here's the good news: Authenticated requests get a higher rate limit. Check out the documentation for more details.) If you reach out to GitHub Support for help, please include the request ID E19B:5D7C:12EAF10:17BA794:67F1D9DD and timestamp 2025-04-06 01:33:17 UTC.","documentation_url":"https://docs.github.com/rest/overview/rate-limits-for-the-rest-api","status":"403"}

message: a notice level event has occurred
severity: NOTICE
Query executed successfully.
```