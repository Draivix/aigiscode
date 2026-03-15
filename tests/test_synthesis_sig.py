import inspect

from aigiscode.synthesis.claude import synthesize


def test_synthesize_accepts_primary_backend():
    sig = inspect.signature(synthesize)
    assert "primary_backend" in sig.parameters
