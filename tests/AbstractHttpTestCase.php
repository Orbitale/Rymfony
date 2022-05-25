<?php

namespace Rymfony\Tests;

use PHPUnit\Framework\TestCase;
use Symfony\Contracts\HttpClient\HttpClientInterface;
use Symfony\Contracts\HttpClient\ResponseInterface;
use function createClient;
use function explode;
use function json_decode;
use function preg_replace;
use function sprintf;
use function trim;

abstract class AbstractHttpTestCase extends TestCase
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

    /**
     * @param ResponseInterface $response
     * @return mixed
     */
    protected static function getJsonFromResponse(ResponseInterface $response): mixed
    {
        $content = $response->getContent();
        self::assertStringStartsWith('Hey! It works!', $content);
        $content = trim(preg_replace('~^Hey! It works!~', '', $content));
        self::assertJson($content);

        return json_decode($content, true);
    }

    protected static function assertArrayPathEquals(array $json, string $path, mixed $expectedValue): void
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

        if ($expectedValue instanceof Regex) {
            self::assertMatchesRegularExpression(
                $expectedValue->toString(),
                $value,
                sprintf('Failed to assert that path "%s" matches regular expression "%s" in JSON array.', $path, $expectedValue->toString())
            );
        } else {
            self::assertSame(
                $expectedValue,
                $value,
                sprintf('Failed to assert that path "%s" is equal to "%s" in JSON array.', $path, self::normalizeValue($expectedValue))
            );
        }
    }

    /**
     * @param array<Regex|mixed> $expectations
     */
    protected static function assertMultipleArrayPathsEqual(array $array, array $expectations): void
    {
        foreach ($expectations as $path => $expectation) {
            self::assertArrayPathEquals($array, $path, $expectation);
        }
    }

    private static function normalizeValue(mixed $value): string
    {
        if (is_scalar($value) || (is_object($value) && method_exists($value, '__toString'))) {
            return (string) $value;
        }

        return print_r($value, true);
    }
}
