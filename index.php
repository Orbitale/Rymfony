<?php

$fields = [
    "CONTENT_LENGTH",
    "CONTENT_TYPE",
    "DOCUMENT_ROOT",
    "DOCUMENT_URI",
    "GATEWAY_INTERFACE",
    "HTTPS",
    "ORIG_PATH_INFO",
    "PATH_INFO",
    "QUERY_STRING",
    "REMOTE_ADDR",
    "REMOTE_HOST",
    "REMOTE_PORT",
    "REQUEST_METHOD",
    "REQUEST_URI",
    "SCRIPT_FILENAME",
    "SCRIPT_NAME",
    "SERVER_ADDR",
    "SERVER_ADMIN",
    "SERVER_NAME",
    "SERVER_PORT",
    "SERVER_PROTOCOL",
    "SERVER_SOFTWARE",
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


header('X-Some-Random-Header: some-random-value');
$content = "Hey! It works!\n"
    .json_encode([
    'Server parameters' => $display,
    'Request headers' => $headers,
    'Request body' => $requestBody,
], JSON_PRETTY_PRINT | JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);

$date = date('Y-m-d H:i:s');

$logs = <<<LOG
===========================
Date: {$date}
Content:
{$content}
===========================

LOG;

file_put_contents('_local_logs.txt', $logs, FILE_APPEND);

echo $content;
