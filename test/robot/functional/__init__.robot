*** Settings ***
Resource          ${CURDIR}/../../../stackql-core/test/robot/functional/stackql.resource
Suite Setup       Prepare StackQL Environment
Suite Teardown    Terminate All Processes    kill=True

