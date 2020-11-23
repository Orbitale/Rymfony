curl -i \
    'http://127.0.0.1:8000/toto?custom_query=string_value#some_fragment' \
    -X POST \
    -d '{"wow": true}' \
    -H "X-Some: Val" \
    -H "Cookie: SomeCookie=some_value"
