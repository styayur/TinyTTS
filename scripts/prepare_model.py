"""Explicit CI/local acceptance input preparation; never called by the application."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import tarfile
import urllib.request

ROOT=Path(__file__).resolve().parents[1]
NAME='kokoro-multi-lang-v1_0'
SIZE=349886476
SHA256='60703707bc05ccd48d60e1eb723f38b07d6e474795a0b6a588d7220400e0f7a4'


def prepare(archive_dir=None):
    destination=ROOT/'.test-model';destination.mkdir(exist_ok=True)
    archive=destination/(NAME+'.tar.bz2')
    if not archive.exists():
        partial=archive.with_suffix('.part')
        if archive_dir:
            shutil.copyfile(Path(archive_dir)/archive.name,partial)
        else:
            url='https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/'+archive.name
            with urllib.request.urlopen(url,timeout=60) as source,partial.open('wb') as output:
                count=0
                while chunk:=source.read(1024*1024):
                    count+=len(chunk)
                    if count>SIZE:raise ValueError('Model archive exceeds pinned size')
                    output.write(chunk)
        partial.replace(archive)
    with archive.open('rb') as source: digest=hashlib.file_digest(source,'sha256').hexdigest()
    if archive.stat().st_size!=SIZE or digest!=SHA256:
        raise ValueError('Model archive identity mismatch; refusing extraction')
    with tarfile.open(archive) as source:
        members=source.getmembers()
        if len(members)>10000 or sum(m.size for m in members)>3*1024**3:raise ValueError('Unexpected model bounds')
        names=set()
        for member in members:
            path=PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or not path.parts or path.parts[0]!=NAME or not(member.isfile() or member.isdir()) or member.name in names:
                raise ValueError('Unsafe model archive member')
            names.add(member.name)
        source.extractall(destination,members=members,filter='data')
    model=destination/NAME
    if (model/'voices.bin').stat().st_size!=54*510*256*4:raise ValueError('Unexpected reviewed voice-table shape')
    (destination/'verification.json').write_text(json.dumps(dict(archive_sha256=digest,size=SIZE,speakers=54,redistributed=False),indent=2)+'\n',encoding='utf-8')
    return model.resolve()


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--archive-dir');parser.add_argument('--github-env',action='store_true')
    args=parser.parse_args();model=prepare(args.archive_dir)
    if args.github_env:
        with Path(os.environ['GITHUB_ENV']).open('a',encoding='utf-8') as stream:stream.write('TINY_TTS_TEST_MODEL='+str(model)+'\n')


if __name__=='__main__':main()
