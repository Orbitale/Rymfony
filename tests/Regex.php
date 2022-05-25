<?php

namespace Rymfony\Tests;

final class Regex
{
    private string $pattern;

    private function __construct(string $pattern)
    {
        $this->pattern = $pattern;
    }

    public function toString(): string
    {
        return $this->pattern;
    }

    public static function from(string $pattern): self
    {
        return new self($pattern);
    }
}
