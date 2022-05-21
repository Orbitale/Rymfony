<?php

namespace Rymfony;

use PHPUnit\Runner\AfterLastTestHook;
use PHPUnit\Runner\BeforeFirstTestHook;
use RuntimeException;
use Symfony\Component\Console\Input\ArgvInput;
use Symfony\Component\Console\Output\ConsoleOutput;
use Symfony\Component\Console\Style\SymfonyStyle;
use Symfony\Component\Process\Process;
use Symfony\Contracts\HttpClient\Exception\TransportExceptionInterface;
use function dirname;
use function file_exists;
use function sprintf;

class HttpServerExtension implements AfterLastTestHook, BeforeFirstTestHook
{
    private ?Process $process = null;
    private SymfonyStyle $io;

    public function __construct()
    {
        $this->io = new SymfonyStyle(new ArgvInput(), new ConsoleOutput());
    }

    public function executeBeforeFirstTest(): void
    {
        $projectDirectory = dirname(__DIR__, 2);
        $rymfonyBinPath = $projectDirectory.'/target/release/rymfony';

        if (!file_exists($rymfonyBinPath)) {
            $rymfonyBinPath .= '.exe';
            if (!file_exists($rymfonyBinPath)) {
                throw new RuntimeException(sprintf(
                    "Rymfony binary is not found in %s.\n".
                    'Did you forget to run "cargo build --release"?\n',
                    pathinfo($rymfonyBinPath, PATHINFO_DIRNAME)
                ));
            }
        }

        $this->io->writeln('<info>Ensuring Rymfony is stopped...</info>');
        $process = new Process([$rymfonyBinPath, 'stop'], $projectDirectory);
        $process->run();

        $process = new Process([$rymfonyBinPath, 'serve'], $projectDirectory);

        $process->start();

        $executionResult = $process->waitUntil(function ($_, string $output)  {
            return str_contains($output, 'Listening to https://127.0.0.1:8000');
        });

        if (!$executionResult) {
            $this->io->error('Rymfony server could not start.');

            throw new RuntimeException();
        }

        $this->io->writeln('<info>Rymfony server started</info>');

        $client = createClient();
        for ($i = 0; $i < 5; $i++) {
            try {
                $res = $client->request('GET', '/');

                if ($res->getStatusCode() === 200) {
                    break;
                }
            } catch (TransportExceptionInterface $e) {
            }
        }

        sleep(1); // Because "wait until" and attempting to make requests is never enough..

        $this->io->writeln('<info>Rymfony server ready</info>');

        $this->process = $process;
    }

    public function executeAfterLastTest(): void
    {
        if ($this->process) {
            $this->process->stop(1);
            $this->process = null;
            echo "\n"; // Because phpunit messes the output sometimes.
            $this->io->writeln('<info>Rymfony server stopped</info>');
        }
    }
}