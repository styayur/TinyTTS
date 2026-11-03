import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec=importlib.util.spec_from_file_location('prepare_model',Path(__file__).resolve().parents[1]/'scripts/prepare_model.py')
model=importlib.util.module_from_spec(spec);spec.loader.exec_module(model)


class ModelInputSecurity(unittest.TestCase):
    def test_changed_or_corrupt_model_cache_cannot_silently_replace_reviewed_model(self):
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary);cache=root/'.test-model';cache.mkdir()
            (cache/(model.NAME+'.tar.bz2')).write_bytes(b'not the reviewed model')
            with patch.object(model,'ROOT',root):
                with self.assertRaisesRegex(ValueError,'identity mismatch'):
                    model.prepare()
            self.assertFalse((cache/model.NAME).exists())


if __name__=='__main__':unittest.main()
