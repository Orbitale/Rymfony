#!/usr/bin/env bats

#TESTS_DIR="$BATS_TEST_DIRNAME"
#PROJECT_DIR="$( realpath "${TESTS_DIR}/../" )"
#RYMFONY_EXECUTABLE_PATH="${PROJECT_DIR}/target/debug/rymfony"

load "helpers/common"

@test "Make sure server is running" {
    run curl --insecure --no-progress-meter -I https://127.0.0.1:8000
    assert_success
    assert_line --partial --index 0 "HTTP/2 200"
    assert_line --partial --index 1 "content-type: text/plain;charset=UTF-8"
    assert_line --partial --index 2 "content-length: 997"
    assert_line --partial --index 3 "x-some-random-header: some-random-value"
    assert_line  --regexp --index 4 "date: [A-Z][a-z]+, [0-9]{1,2} [A-Z][a-z]+ [0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2} GMT"
    assert_line  --regexp --index 5 "^\s+$"
}

@test "Test output with custom headers" {
    run curl --insecure -i --no-progress-meter \
      https://127.0.0.1:8000/api/project \
      --header 'Accept: application/json' \
      --header 'User-Agent: test runner' \
      --header 'Cookie: test1=1; test2=2' \
      --header 'Authorization: Bearer 6yVnP0Nf6FP025Dby56sxjU'
    assert_success
    assert_line --partial --index 0 "HTTP/2 200"
    assert_line --partial --index 1 "content-type: text/plain;charset=UTF-8"
    assert_line --partial --index 2 "content-length: 1150"
    assert_line --partial --index 3 "x-some-random-header: some-random-value"
    assert_line  --regexp --index 4 "date: [A-Z][a-z]+, [0-9]{1,2} [A-Z][a-z]+ [0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2} GMT"
    assert_line  --regexp --index 5 "^\s+$"
    assert_line --partial --index 6 'Hey! It works!'
    assert_line --partial --index 7 '{'
    assert_line  --regexp --index 8 '    "Date": "[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}",'
    assert_line --partial --index 9 '    "Server parameters": {'
    assert_line --partial --index 10 '        "CONTENT_LENGTH": "",'
    assert_line --partial --index 11 '        "CONTENT_TYPE": "",'
    assert_line  --regexp --index 12 '        "DOCUMENT_ROOT": "[^"]+",'
    assert_line --partial --index 13 '        "DOCUMENT_URI": "/api/project",'
    assert_line --partial --index 14 '        "GATEWAY_INTERFACE": "FastCGI/1.0",'
    assert_line --partial --index 15 '        "HTTPS": "On",'
    assert_line --partial --index 16 '        "ORIG_PATH_INFO": "__",'
    assert_line --partial --index 17 '        "PATH_INFO": "/api/project",'
    assert_line --partial --index 18 '        "QUERY_STRING": "",'
    assert_line --partial --index 19 '        "REMOTE_ADDR": "127.0.0.1",'
    assert_line --partial --index 20 '        "REMOTE_HOST": "__",'
    assert_line  --regexp --index 21 '        "REMOTE_PORT": "600[0-9]+",'
    assert_line --partial --index 22 '        "REQUEST_METHOD": "GET",'
    assert_line --partial --index 23 '        "REQUEST_URI": "/api/project",'
    assert_line  --regexp --index 24 '        "SCRIPT_FILENAME": "[^"]+/index.php",'
    assert_line --partial --index 25 '        "SCRIPT_NAME": "/index.php",'
    assert_line --partial --index 26 '        "SERVER_ADDR": "__",'
    assert_line --partial --index 27 '        "SERVER_ADMIN": "__",'
    assert_line --partial --index 28 '        "SERVER_NAME": "127.0.0.1:8000",'
    assert_line --partial --index 29 '        "SERVER_PORT": "8000",'
    assert_line --partial --index 30 '        "SERVER_PROTOCOL": "HTTP/1.1",'
    assert_line --partial --index 31 '        "SERVER_SOFTWARE": "Rymfony"'
    assert_line --partial --index 32 '    },'
    assert_line --partial --index 33 '    "Request headers": {'
    assert_line --partial --index 34 '        "HTTP_ACCEPT": "application/json",'
    assert_line --partial --index 35 '        "HTTP_AUTHORIZATION": "Bearer 6yVnP0Nf6FP025Dby56sxjU",'
    assert_line --partial --index 36 '        "HTTP_COOKIE": "test1=1; test2=2",'
    assert_line --partial --index 37 '        "HTTP_HOST": "127.0.0.1:8000",'
    assert_line --partial --index 38 '        "HTTP_USER_AGENT": "test runner"'
    assert_line --partial --index 39 '    },'
    assert_line --partial --index 40 '    "Request body": ""'
    assert_line --partial --index 41 '}'
}

@test "Test output with custom query and fragment" {
    run curl --insecure -i --no-progress-meter \
      https://127.0.0.1:8000/some_path?custom_query=string_value#some_fragment
    assert_success
    assert_line --partial --index 0 "HTTP/2 200"
    assert_line --partial --index 1 "content-type: text/plain;charset=UTF-8"
    assert_line --partial --index 2 "content-length: 1101"
    assert_line --partial --index 3 "x-some-random-header: some-random-value"
    assert_line  --regexp --index 4 "date: [A-Z][a-z]+, [0-9]{1,2} [A-Z][a-z]+ [0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2} GMT"
    assert_line  --regexp --index 5 "^\s+$"
    assert_line --partial --index 6 'Hey! It works!'
    assert_line --partial --index 7 '{'
    assert_line  --regexp --index 8 '    "Date": "[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}",'
    assert_line --partial --index 9 '    "Server parameters": {'
    assert_line --partial --index 10 '        "CONTENT_LENGTH": "",'
    assert_line --partial --index 11 '        "CONTENT_TYPE": "",'
    assert_line  --regexp --index 12 '        "DOCUMENT_ROOT": "[^"]+",'
    assert_line --partial --index 13 '        "DOCUMENT_URI": "/some_path",'
    assert_line --partial --index 14 '        "GATEWAY_INTERFACE": "FastCGI/1.0",'
    assert_line --partial --index 15 '        "HTTPS": "On",'
    assert_line --partial --index 16 '        "ORIG_PATH_INFO": "__",'
    assert_line --partial --index 17 '        "PATH_INFO": "/some_path?custom_query=string_value",'
    assert_line --partial --index 18 '        "QUERY_STRING": "custom_query=string_value",'
    assert_line --partial --index 19 '        "REMOTE_ADDR": "127.0.0.1",'
    assert_line --partial --index 20 '        "REMOTE_HOST": "__",'
    assert_line  --regexp --index 21 '        "REMOTE_PORT": "600[0-9]+",'
    assert_line --partial --index 22 '        "REQUEST_METHOD": "GET",'
    assert_line --partial --index 23 '        "REQUEST_URI": "/some_path?custom_query=string_value",'
    assert_line  --regexp --index 24 '        "SCRIPT_FILENAME": "[^"]+/index.php",'
    assert_line --partial --index 25 '        "SCRIPT_NAME": "/index.php",'
    assert_line --partial --index 26 '        "SERVER_ADDR": "__",'
    assert_line --partial --index 27 '        "SERVER_ADMIN": "__",'
    assert_line --partial --index 28 '        "SERVER_NAME": "127.0.0.1:8000",'
    assert_line --partial --index 29 '        "SERVER_PORT": "8000",'
    assert_line --partial --index 30 '        "SERVER_PROTOCOL": "HTTP/1.1",'
    assert_line --partial --index 31 '        "SERVER_SOFTWARE": "Rymfony"'
    assert_line --partial --index 32 '    },'
    assert_line --partial --index 33 '    "Request headers": {'
    assert_line --partial --index 34 '        "HTTP_ACCEPT": "*/*",'
    assert_line --partial --index 35 '        "HTTP_HOST": "127.0.0.1:8000",'
    assert_line  --regexp --index 36 '        "HTTP_USER_AGENT": "curl/[^"]+"'
    assert_line --partial --index 37 '    },'
    assert_line --partial --index 38 '    "Request body": ""'
    assert_line --partial --index 39 '}'
}
