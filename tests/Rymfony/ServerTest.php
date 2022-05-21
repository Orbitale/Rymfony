<?php

namespace Rymfony;

use function dirname;
use function json_decode;
use function preg_replace;
use function trim;

class ServerTest extends HttpTestCase
{
    public function testDefaultRequest(): void
    {
        $response = $this->client->request('GET', '/');

        self::assertEquals(200, $response->getStatusCode());

        $content = $response->getContent();
        self::assertStringStartsWith('Hey! It works!', $content);
        $content = trim(preg_replace('~^Hey! It works!~', '', $content));
        self::assertJson($content);
        $json = json_decode($content, true);

        $projectDir = dirname(__DIR__, 2);

        self::assertJsonPathEquals($json, 'Server parameters.DOCUMENT_ROOT', $projectDir);
        self::assertJsonPathEquals($json, 'Server parameters.DOCUMENT_URI', '/index.php');
        self::assertJsonPathEquals($json, 'Server parameters.SCRIPT_FILENAME', $projectDir.'/index.php');
        self::assertJsonPathEquals($json, 'Server parameters.SCRIPT_NAME', '/index.php');
        self::assertJsonPathEquals($json, 'Server parameters.SERVER_NAME', '127.0.0.1');
        self::assertJsonPathEquals($json, 'Server parameters.REMOTE_ADDR', '127.0.0.1');
        self::assertJsonPathEquals($json, 'Server parameters.REMOTE_HOST', '127.0.0.1');
        self::assertJsonPathEquals($json, 'Server parameters.SERVER_PORT', '8000');
        self::assertJsonPathEquals($json, 'Server parameters.SERVER_PROTOCOL', 'HTTP/2.0');
        self::assertJsonPathEquals($json, 'Server parameters.SERVER_SOFTWARE', 'Rymfony/Caddy');
        self::assertJsonPathEquals($json, 'Server parameters.HTTPS', 'on');
        self::assertJsonPathEquals($json, 'Server parameters.REQUEST_URI', '/');
        self::assertJsonPathEquals($json, 'Server parameters.CONTENT_LENGTH', '0');
        self::assertJsonPathEquals($json, 'Server parameters.CONTENT_TYPE', '');
        self::assertJsonPathEquals($json, 'Server parameters.QUERY_STRING', '');
        self::assertJsonPathEquals($json, 'Server parameters.PATH_INFO', '');

        self::assertJsonPathEquals($json, 'Request headers.HTTP_ACCEPT', '*/*');
        self::assertJsonPathEquals($json, 'Request headers.HTTP_ACCEPT_ENCODING', 'gzip');
        self::assertJsonPathEquals($json, 'Request headers.HTTP_HOST', '127.0.0.1:8000');
        self::assertJsonPathEquals($json, 'Request headers.HTTP_USER_AGENT', 'Symfony HttpClient/Curl');
        self::assertJsonPathEquals($json, 'Request headers.HTTP_X_FORWARDED_FOR', '127.0.0.1');
        self::assertJsonPathEquals($json, 'Request headers.HTTP_X_FORWARDED_PROTO', 'https');
    }
}
