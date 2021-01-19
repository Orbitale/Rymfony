<?php

declare(strict_types=1);

header("Content-Type: application/pdf");

echo file_get_contents("binaryfile.php.pdf");
