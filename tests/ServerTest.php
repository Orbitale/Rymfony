<?php

namespace Rymfony\Tests;

use function dirname;
use function fclose;
use function file_get_contents;
use function fopen;
use function json_decode;
use function preg_replace;
use function trim;

class ServerTest extends AbstractHttpTestCase
{
    public function testDefaultRequest(): void
    {
        $response = $this->client->request('GET', '/');

        self::assertEquals(200, $response->getStatusCode());

        $json = self::getJsonFromResponse($response);

        $projectDir = dirname(__DIR__);

        self::assertMultipleArrayPathsEqual($json, [
            'Server parameters.DOCUMENT_ROOT' => $projectDir,
            'Server parameters.DOCUMENT_URI' => Regex::from('~^/?index.php$~'),
            'Server parameters.SCRIPT_FILENAME' => $projectDir.'/index.php',
            'Server parameters.SCRIPT_NAME' => '/index.php',
            'Server parameters.SERVER_NAME' => '127.0.0.1',
            'Server parameters.REMOTE_ADDR' => '127.0.0.1',
            'Server parameters.REMOTE_HOST' => '127.0.0.1',
            'Server parameters.SERVER_PORT' => '8000',
            'Server parameters.SERVER_PROTOCOL' => 'HTTP/2.0',
            'Server parameters.SERVER_SOFTWARE' => 'Rymfony/Caddy',
            'Server parameters.HTTPS' => 'on',
            'Server parameters.REQUEST_URI' => '/',
            'Server parameters.CONTENT_LENGTH' => '0',
            'Server parameters.CONTENT_TYPE' => '',
            'Server parameters.QUERY_STRING' => '',
            'Server parameters.PATH_INFO' => '',

            'Request headers.HTTP_ACCEPT' => '*/*',
            'Request headers.HTTP_ACCEPT_ENCODING' => 'gzip',
            'Request headers.HTTP_HOST' => '127.0.0.1:8000',
            'Request headers.HTTP_USER_AGENT' => 'Symfony HttpClient/Curl',
            'Request headers.HTTP_X_FORWARDED_FOR' => '127.0.0.1',
            'Request headers.HTTP_X_FORWARDED_PROTO' => 'https',
        ]);
    }

    public function testWithCustomRequestHeaders(): void
    {
        $response = $this->client->request('GET', '/headers-request', [
            'headers' => [
                'Cookie' => 'test1=1; test2=2',
                'Authorization' => 'Bearer 1628e59843ef661628e59843ef68',
            ],
        ]);

        self::assertEquals(200, $response->getStatusCode());

        $json = self::getJsonFromResponse($response);

        self::assertMultipleArrayPathsEqual($json, [
            'Server parameters.REQUEST_URI' => '/headers-request',

            'Request headers.HTTP_COOKIE' => 'test1=1; test2=2',
            'Request headers.HTTP_AUTHORIZATION' => 'Bearer 1628e59843ef661628e59843ef68',
        ]);
    }

    public function testPostWithFormBody(): void
    {
        $response = $this->client->request('POST', '/post-request-form', [
            'body' => 'field1=value1&field2=value2',
            'headers' => [
                'Content-Type' => 'application/x-www-form-urlencoded',
            ],
        ]);

        self::assertEquals(200, $response->getStatusCode());

        $json = self::getJsonFromResponse($response);

        self::assertMultipleArrayPathsEqual($json, [
            'Server parameters.DOCUMENT_URI' => Regex::from('~^/?index.php$~'),
            'Server parameters.REQUEST_METHOD' => 'POST',
            'Server parameters.REQUEST_URI' => '/post-request-form',
            'Server parameters.CONTENT_LENGTH' => '27',
            'Server parameters.CONTENT_TYPE' => 'application/x-www-form-urlencoded',
            'Request body' => 'field1=value1&field2=value2',
        ]);
    }

    public function testPostWithJsonBody(): void
    {
        $response = $this->client->request('POST', '/post-request-json', [
            'body' => '{"field1":"value1","field2":"value2"}',
            'headers' => [
                'Content-Type' => 'application/json',
            ],
        ]);

        self::assertEquals(200, $response->getStatusCode());

        $json = self::getJsonFromResponse($response);

        self::assertMultipleArrayPathsEqual($json, [
            'Server parameters.DOCUMENT_URI' => Regex::from('~^/?index.php$~'),
            'Server parameters.REQUEST_METHOD' => 'POST',
            'Server parameters.REQUEST_URI' => '/post-request-json',
            'Server parameters.CONTENT_LENGTH' => '37',
            'Server parameters.CONTENT_TYPE' => 'application/json',
            'Request body' => '{"field1":"value1","field2":"value2"}',
        ]);
    }

    public function testGetAnotherFile(): void
    {
        $response = $this->client->request('GET', '/tests/php_serve_binary/index.php');

        self::assertEquals(200, $response->getStatusCode());

        $headers = $response->getHeaders();

        self::assertMultipleArrayPathsEqual($headers, [
            'content-length' => ['9017'],
            'content-type' => ['application/pdf'],
        ]);
    }
}
