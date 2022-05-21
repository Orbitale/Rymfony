<?php

namespace Rymfony;

use PHPUnit\Framework\TestCase;
use Symfony\Contracts\HttpClient\HttpClientInterface;

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
}