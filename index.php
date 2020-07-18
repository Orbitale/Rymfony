<?php

echo "Hey! It works!\n";

$fields = [
    "DOCUMENT_ROOT",
    "HTTPS",
    "ORIG_PATH_INFO",
    "PATH_INFO",
    "QUERY_STRING",
    "REMOTE_ADDR",
    "REMOTE_HOST",
    "REQUEST_URI",
    "SCRIPT_FILENAME",
    "SCRIPT_NAME",
    "SERVER_ADMIN",
    "SERVER_PORT",
    "SERVER_PORT",
];

foreach ($fields as $field) {
    $display[$field] = $_SERVER[$field] ?? '__undefined';
}

$headers = [];

foreach ($_SERVER as $key => $value) {
    if (0 === strpos($key, 'HTTP_')) {
        $headers[$key] = $value;
    }
}

ksort($headers);
ksort($display);

echo json_encode([
    'Server' => $display,
    'Headers' => $headers,
], JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
