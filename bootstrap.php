<?php

use Symfony\Component\HttpClient\HttpClient;
use Symfony\Contracts\HttpClient\HttpClientInterface;

require __DIR__.'/vendor/autoload.php';

const RYMFONY_BASE_URI = 'https://127.0.0.1:8000';

function createClient(): HttpClientInterface
{
    return HttpClient::create([
        'base_uri' => RYMFONY_BASE_URI,
        'http_version' => '2.0',
    ]);
}
