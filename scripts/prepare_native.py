"""Prepare hash-pinned Windows native inputs independently of Cargo target caches."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import tarfile
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
NAME = 'sherpa-onnx-v1.13.8-win-x64-static-MT-Release-lib'
SHA256 = '56ffcf3c454c1f14f7bc9887286cc8143e7e542dc632804e1c447d5f8d534eaf'
SIZE = 123206268
LIBRARIES = ['sherpa-onnx-c-api', 'sherpa-onnx-core', 'kaldi-decoder-core',
             'sherpa-onnx-kaldifst-core', 'sherpa-onnx-fstfar', 'sherpa-onnx-fst',
             'kaldi-native-fbank-core', 'kissfft-float', 'piper_phonemize',
             'espeak-ng', 'ucd', 'onnxruntime', 'ssentencepiece_core']


def prepare(archive_dir=None):
    lock = tomllib.loads((ROOT / 'Cargo.lock').read_text(encoding='utf-8'))
    native = next(p for p in lock['package'] if p['name'] == 'sherpa-onnx-sys')
    if native['version'] != '1.13.8':
        raise ValueError('Review native archive identity for the new sherpa-onnx version')
    destination = ROOT / '.native'
    destination.mkdir(exist_ok=True)
    archive = destination / (NAME + '.tar.bz2')
    if not archive.exists():
        partial = archive.with_suffix('.part')
        if archive_dir:
            shutil.copyfile(Path(archive_dir) / archive.name, partial)
        else:
            url = f'https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.8/{archive.name}'
            with urllib.request.urlopen(url, timeout=60) as response, partial.open('wb') as output:
                count = 0
                while chunk := response.read(1024 * 1024):
                    count += len(chunk)
                    if count > SIZE:
                        raise ValueError('Native archive exceeds pinned size')
                    output.write(chunk)
        partial.replace(archive)
    with archive.open('rb') as source:
        digest = hashlib.file_digest(source, 'sha256').hexdigest()
    if archive.stat().st_size != SIZE or digest != SHA256:
        raise ValueError('Native archive identity mismatch; refusing extraction')
    with tarfile.open(archive) as source:
        members = source.getmembers()
        if len(members) > 100 or sum(m.size for m in members) > 2 * 1024**3:
            raise ValueError('Unexpected native archive bounds')
        for member in members:
            path = PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or not path.parts or path.parts[0] != NAME or not (member.isfile() or member.isdir()):
                raise ValueError('Unsafe native archive member')
        # Rehydrate the inputs every run. A cached build-script output can refer
        # to a library path that Cargo's ordinary target cache did not restore.
        source.extractall(destination, members=members, filter='data')
    lib_dir = (destination / NAME / 'lib').resolve()
    for name in LIBRARIES:
        if not (lib_dir / (name + '.lib')).is_file():
            raise ValueError('Missing native library: ' + name)
    (destination / 'verification.json').write_text(json.dumps(dict(version=native['version'], archive_sha256=digest, libraries=LIBRARIES), indent=2) + '\n', encoding='utf-8')
    return lib_dir


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--archive-dir')
    parser.add_argument('--github-env', action='store_true')
    args = parser.parse_args()
    lib_dir = prepare(args.archive_dir)
    if args.github_env:
        with Path(os.environ['GITHUB_ENV']).open('a', encoding='utf-8') as output:
            output.write('SHERPA_ONNX_LIB_DIR=' + str(lib_dir) + '\n')


if __name__ == '__main__':
    main()
