*** Settings ***
Resource          ${CURDIR}/stackql.resource

*** Test Cases ***

PG Session Debug Behaviour Canonical
    [Documentation]   The mTLS servers have debug on
    Set Environment Variable    PGHOST           ${PSQL_CLIENT_HOST}
    Set Environment Variable    PGPORT           ${PG_SRV_PORT_MTLS}
    Set Environment Variable    PGSSLMODE        verify\-full 
    Set Environment Variable    PGSSLCERT        ${STACKQL_PG_CLIENT_CERT_PATH} 
    Set Environment Variable    PGSSLKEY         ${STACKQL_PG_CLIENT_KEY_PATH} 
    Set Environment Variable    PGSSLROOTCERT    ${STACKQL_PG_SERVER_CERT_PATH} 
    ${inputStr} =    Catenate
    ...    SELECT repo, count(1) as has_starred
    ...    FROM github.activity.repo_stargazers    
    ...    WHERE owner = 'sillyorg' and repo in ('silly', 'silly-but-more') and login = 'sillylogin'
    ...    GROUP BY repo;
    ${outputStr} =    Catenate    SEPARATOR=\n
    ...    repo | has_starred 
    ...    ------+-------------
    ...    (0 rows)
    ${outputErrStr} =    Catenate    SEPARATOR=\n
    ...    NOTICE:  a notice level event has occurred
    ...    DETAIL:  http response status code: 403, response body: "{\\"message\\":\\"API rate limit exceeded for 111.1111.111.11. (But here's the good news: Authenticated requests get a higher rate limit. Check out the documentation for more details.) If you reach out to GitHub Support for help, please include the request ID AAAA:AAAA:BBBBBB:BBBBBB:BBBBBBBB and timestamp 2025-01-01 01:00:00 UTC.\\",\\"documentation_url\\":\\"https://docs.github.com/rest/overview/rate-limits-for-the-rest-api\\",\\"status\\":\\"403\\"}"
    ...    
    ...    http response status code: 403, response body: "{\\"message\\":\\"API rate limit exceeded for 111.1111.111.11. (But here's the good news: Authenticated requests get a higher rate limit. Check out the documentation for more details.) If you reach out to GitHub Support for help, please include the request ID AAAA:AAAA:BBBBBB:BBBBBB:BBBBBBBB and timestamp 2025-01-01 01:00:00 UTC.\\",\\"documentation_url\\":\\"https://docs.github.com/rest/overview/rate-limits-for-the-rest-api\\",\\"status\\":\\"403\\"}" 
    ${posixInput} =     Catenate
    ...    "${PSQL_EXE}" -c "${inputStr}"
    ${windowsInput} =     Catenate
    ...    &    ${posixInput}
    ${input} =    Set Variable If    "${IS_WINDOWS}" == "1"    ${windowsInput}    ${posixInput}
    ${shellExe} =    Set Variable If    "${IS_WINDOWS}" == "1"    powershell    sh
    ${result} =    Run Process
    ...    ${shellExe}     \-c    ${input}
    ...    stdout=${CURDIR}/tmp/PG-Session-Debug-Behaviour-Canonical.tmp
    ...    stderr=${CURDIR}/tmp/PG-Session-Debug-Behaviour-Canonical-stderr.tmp
    Log    STDOUT = "${result.stdout}"
    Log    STDERR = "${result.stderr}"
    # Should Contain    ${result.stdout}    ${outputStr}    collapse_spaces=True
    Should Contain    ${result.stderr}    ${outputErrStr}    collapse_spaces=True
    [Teardown]  Run Keywords    Remove Environment Variable     PGHOST
    ...         AND             Remove Environment Variable     PGPORT
    ...         AND             Remove Environment Variable     PGSSLMODE 
    ...         AND             Remove Environment Variable     PGSSLCERT 
    ...         AND             Remove Environment Variable     PGSSLKEY
    ...         AND             Remove Environment Variable     PGSSLROOTCERT

PG Session Debug On Unencryopted Server Behaviour Canonical
    [Documentation]   The unencrypted servers also have debug on
    Set Environment Variable    PGHOST           ${PSQL_CLIENT_HOST}
    Set Environment Variable    PGPORT           ${PG_SRV_PORT_UNENCRYPTED}
    Set Environment Variable    PGUSER           stackql
    Set Environment Variable    PGPASSWORD       ${PSQL_PASSWORD} 
    ${inputStr} =    Catenate
    ...    SELECT repo, count(1) as has_starred
    ...    FROM github.activity.repo_stargazers    
    ...    WHERE owner = 'sillyorg' and repo in ('silly', 'silly-but-more') and login = 'sillylogin'
    ...    GROUP BY repo;
    ${outputStr} =    Catenate    SEPARATOR=\n
    ...    repo | has_starred 
    ...    ------+-------------
    ...    (0 rows)
    ${outputErrStr} =    Catenate    SEPARATOR=\n
    ...    NOTICE:  a notice level event has occurred
    ...    DETAIL:  http response status code: 403, response body: "{\\"message\\":\\"API rate limit exceeded for 111.1111.111.11. (But here's the good news: Authenticated requests get a higher rate limit. Check out the documentation for more details.) If you reach out to GitHub Support for help, please include the request ID AAAA:AAAA:BBBBBB:BBBBBB:BBBBBBBB and timestamp 2025-01-01 01:00:00 UTC.\\",\\"documentation_url\\":\\"https://docs.github.com/rest/overview/rate-limits-for-the-rest-api\\",\\"status\\":\\"403\\"}"
    ...    
    ...    http response status code: 403, response body: "{\\"message\\":\\"API rate limit exceeded for 111.1111.111.11. (But here's the good news: Authenticated requests get a higher rate limit. Check out the documentation for more details.) If you reach out to GitHub Support for help, please include the request ID AAAA:AAAA:BBBBBB:BBBBBB:BBBBBBBB and timestamp 2025-01-01 01:00:00 UTC.\\",\\"documentation_url\\":\\"https://docs.github.com/rest/overview/rate-limits-for-the-rest-api\\",\\"status\\":\\"403\\"}" 
    ${posixInput} =     Catenate
    ...    "${PSQL_EXE}" -c "${inputStr}"
    ${windowsInput} =     Catenate
    ...    &    ${posixInput}
    ${input} =    Set Variable If    "${IS_WINDOWS}" == "1"    ${windowsInput}    ${posixInput}
    ${shellExe} =    Set Variable If    "${IS_WINDOWS}" == "1"    powershell    sh
    ${result} =    Run Process
    ...    ${shellExe}     \-c    ${input}
    ...    stdout=${CURDIR}/tmp/PG-Session-Debug-On-Unencrypted-Server-Behaviour-Canonical.tmp
    ...    stderr=${CURDIR}/tmp/PG-Session-Debug-On-Unencrypted-Server-Behaviour-Canonical-stderr.tmp
    Log    STDOUT = "${result.stdout}"
    Log    STDERR = "${result.stderr}"
    # Should Contain     ${result.stdout}    ${outputStr}   collapse_spaces=True
    Should Contain    ${result.stderr}    ${outputErrStr}    collapse_spaces=True
    [Teardown]  Run Keywords    Remove Environment Variable     PGHOST
    ...         AND             Remove Environment Variable     PGPORT
    ...         AND             Remove Environment Variable     PGUSER 
    ...         AND             Remove Environment Variable     PGPASSWORD

Rust Testing Client Positive Control Notice Messages from GitHub
    [Documentation]   The unencrypted servers also have debug on
    Set Environment Variable    PGHOST           ${PSQL_CLIENT_HOST}
    Set Environment Variable    PGPORT           ${PG_SRV_PORT_UNENCRYPTED}
    Set Environment Variable    PGUSER           stackql
    Set Environment Variable    PGPASSWORD       ${PSQL_PASSWORD} 
    ${inputStr} =    Catenate
    ...    SELECT repo, count(1) as has_starred
    ...    FROM github.activity.repo_stargazers    
    ...    WHERE owner = 'sillyorg' and repo in ('silly', 'silly-but-more') and login = 'sillylogin'
    ...    GROUP BY repo;
    ${posixInput} =     Catenate
    ...    "${RUST_TESTING_EXE}" "${inputStr}" "host=localhost port=${PG_SRV_PORT_UNENCRYPTED}"
    ${windowsInput} =     Catenate
    ...    &    ${posixInput}
    ${input} =    Set Variable If    "${IS_WINDOWS}" == "1"    ${windowsInput}    ${posixInput}
    ${shellExe} =    Set Variable If    "${IS_WINDOWS}" == "1"    powershell    sh
    ${outputErrStrFragment} =    Catenate    SEPARATOR=\n
    ...    http response status code: 403
    ${result} =    Run Process
    ...    ${shellExe}     \-c    ${input}
    ...    stdout=${CURDIR}/tmp/Rust-Testing-Client-Positive-Control-Notice-Messages-from-GitHub.tmp
    ...    stderr=${CURDIR}/tmp/Rust-Testing-Client-Positive-Control-Notice-Messages-from-GitHub-stderr.tmp
    Log    STDOUT = "${result.stdout}"
    Log    STDERR = "${result.stderr}"
    Should Be Equal    ${result.rc}    0
    Should Contain    ${result.stderr}    ${outputErrStrFragment}    collapse_spaces=True
    [Teardown]  Run Keywords    Remove Environment Variable     PGHOST
    ...         AND             Remove Environment Variable     PGPORT
    ...         AND             Remove Environment Variable     PGUSER 
    ...         AND             Remove Environment Variable     PGPASSWORD
