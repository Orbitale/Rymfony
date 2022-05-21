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

        self::assertSame($projectDir, $json['Server parameters']['DOCUMENT_ROOT']);
        self::assertSame('/index.php', $json['Server parameters']['DOCUMENT_URI']);
        self::assertSame($projectDir.'/index.php', $json['Server parameters']['SCRIPT_FILENAME']);
        self::assertSame('index.php', $json['Server parameters']['SCRIPT_NAME']);
        self::assertSame('127.0.0.1', $json['Server parameters']['SERVER_NAME']);
        self::assertSame('127.0.0.1', $json['Server parameters']['REMOTE_ADDR']);
        self::assertSame('127.0.0.1', $json['Server parameters']['REMOTE_HOST']);
        self::assertSame('8000', $json['Server parameters']['SERVER_PORT']);
        self::assertSame('HTTP/2', $json['Server parameters']['SERVER_PROTOCOL']);
        self::assertSame('Rymfony/Caddy', $json['Server parameters']['SERVER_SOFTWARE']);
        self::assertSame('on', $json['Server parameters']['HTTPS']);
        self::assertSame('/', $json['Server parameters']['REQUEST_URI']);
        self::assertSame('0', $json['Server parameters']['CONTENT_LENGTH']);
        self::assertSame('0', $json['Server parameters']['CONTENT_TYPE']);
        self::assertSame('', $json['Server parameters']['QUERY_STRING']);
        self::assertSame('', $json['Server parameters']['PATH_INFO']);

        self::assertSame('*/*', $json['Request headers']['HTTP_ACCEPT']);
        self::assertSame('gzip', $json['Request headers']['HTTP_ACCEPT_ENCODING']);
        self::assertSame('127.0.0.1:8000', $json['Request headers']['HTTP_HOST']);
        self::assertSame('Symfony HttpClient/Curl', $json['Request headers']['HTTP_USER_AGENT']);
        self::assertSame('127.0.0.1', $json['Request headers']['HTTP_X_FORWARDED_FOR']);
        self::assertSame('http', $json['Request headers']['HTTP_X_FORWARDED_PROTO']);
    }
}
