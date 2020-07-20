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
    "REQUEST_METHOD",
    "SCRIPT_FILENAME",
    "SCRIPT_NAME",
    "SERVER_ADMIN",
    "SERVER_PORT",
    "SERVER_PORT",
];

foreach ($fields as $field) {
    $display[$field] = $_SERVER[$field] ?? "__";
}

$headers = [];

foreach ($_SERVER as $key => $value) {
    if (0 === strpos($key, 'HTTP_')) {
        $headers[$key] = $value;
    }
}

ksort($headers);
ksort($display);

$requestBody = file_get_contents("php://input");

echo json_encode([
    'Server parameters' => $display,
    'Request headers' => $headers,
    'Request body' => $requestBody,
], JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
