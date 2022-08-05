<?php

declare(strict_types=1);

use Symfony\Component\HttpFoundation\BinaryFileResponse;
use Symfony\Component\HttpFoundation\Request;

require __DIR__.'/../../vendor/autoload.php';

$request = Request::createFromGlobals();
$response = new BinaryFileResponse(__DIR__.'/binaryfile.pdf');

$response->prepare($request);
$response->send();
