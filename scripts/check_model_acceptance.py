"""Run only the audit-owned native probe, then independently inspect real PCM files."""
from array import array
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import tomllib
import wave

ROOT=Path(__file__).resolve().parents[1]
exe=ROOT/'target/debug/examples'/('native_model_acceptance.exe' if os.name=='nt' else 'native_model_acceptance')
result=subprocess.run([str(exe)],cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=180)
output=ROOT/'work/model-acceptance'
(output/'native-run.log').write_text(result.stdout+'\n'+result.stderr,encoding='utf-8')
if result.returncode:raise RuntimeError('Native probe failed; see work/model-acceptance/native-run.log')
if 'OOV word skipped' in result.stderr:raise AssertionError('Native runtime skipped input words; non-silent PCM is not compatible speech acceptance')
report=tomllib.loads((output/'native-report.toml').read_text(encoding='utf-8'))
assert report['speakers']==54 and report['missing_model_rejected'] and report['pre_cancelled_rejected'] and report['in_flight_cancelled']
assert [case['name'] for case in report['cases']]==['english','english-fast','chinese','spanish-new-speaker','english-unknown-names']
for case in report['cases']:
    file=output/(case['name']+'.wav')
    with wave.open(str(file),'rb') as audio:
        assert (audio.getnchannels(),audio.getsampwidth(),audio.getframerate())==(1,2,24000)
        assert audio.getnframes()==case['frames']
        samples=array('h');samples.frombytes(audio.readframes(audio.getnframes()))
    rms=math.sqrt(sum((sample/32768)**2 for sample in samples)/len(samples))
    assert 0.001<rms<0.9
    case.update(wav_sha256=hashlib.sha256(file.read_bytes()).hexdigest(),seconds=len(samples)/24000,pcm_rms=rms)
report.update(model_sha256='60703707bc05ccd48d60e1eb723f38b07d6e474795a0b6a588d7220400e0f7a4',paid_or_remote_model_calls=0,synthetic_text_only=True,limits=['No independent linguistic-intelligibility assessment','Physical output-device and GUI acceptance are separate local checks; this process verifies actual native inference and PCM exports.'])
(output/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
