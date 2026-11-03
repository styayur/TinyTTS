import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('prepare_native', Path(__file__).resolve().parents[1] / 'scripts/prepare_native.py')
native = importlib.util.module_from_spec(spec)
spec.loader.exec_module(native)


class NativeInputSecurity(unittest.TestCase):
    def test_corrupt_cached_archive_is_rejected_before_extraction(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'Cargo.lock').write_text('[[package]]\nname = "sherpa-onnx-sys"\nversion = "1.13.8+tinytts.zip9.1"\n', encoding='utf-8')
            cache = root / '.native'
            cache.mkdir()
            (cache / (native.NAME + '.tar.bz2')).write_bytes(b'untrusted cache entry')
            sentinel = root / 'sentinel.txt'
            sentinel.write_text('preserve', encoding='utf-8')
            with patch.object(native, 'ROOT', root):
                with self.assertRaisesRegex(ValueError, 'identity mismatch'):
                    native.prepare()
            self.assertFalse((cache / native.NAME).exists())
            self.assertEqual(sentinel.read_text(encoding='utf-8'), 'preserve')

    def test_unreviewed_runtime_version_cannot_reuse_old_native_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'Cargo.lock').write_text('[[package]]\nname = "sherpa-onnx-sys"\nversion = "2.0.0"\n', encoding='utf-8')
            with patch.object(native, 'ROOT', root):
                with self.assertRaisesRegex(ValueError, 'Review native archive identity'):
                    native.prepare()
            self.assertFalse((root / '.native').exists())


if __name__ == '__main__':
    unittest.main()
