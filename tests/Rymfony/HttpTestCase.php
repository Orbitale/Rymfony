<?php

namespace Rymfony;

use PHPUnit\Framework\TestCase;
use Symfony\Contracts\HttpClient\HttpClientInterface;
use function createClient;
use function explode;
use function sprintf;

abstract class HttpTestCase extends TestCase
{
    protected ?HttpClientInterface $client = null;

    protected function setUp(): void
    {
        parent::setUp();
        $this->client = createClient();
    }

    protected function tearDown(): void
    {
        parent::tearDown();
        $this->client = null;
    }

    protected static function assertJsonPathEquals(array $json, string $path, string|int|bool|null $expectedValue): void
    {
        $pathKeys = explode('.', $path);

        $value = $json;

        $visitedPath = '';

        foreach ($pathKeys as $key) {
            $visitedPath .= ($visitedPath ? '.' : '').$key;

            if (!array_key_exists($key, $value)) {
                self::fail(sprintf('Failed to assert that path "%s" is found in JSON array.', $visitedPath));
            }

            $value = $value[$key];
        }

        self::assertSame($expectedValue, $value, sprintf('Failed to assert that path "%s" is equal to "%s" in JSON array.', $path, $expectedValue));
    }
}