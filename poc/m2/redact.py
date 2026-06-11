"""redact(s): ASCII strings pass through; others become <ja len hash8>."""
import hashlib


def redact(s):
    if all(ord(c) < 0x80 for c in s):
        return s
    return "<ja {} {}>".format(len(s), hashlib.sha256(s.encode()).digest()[:8].hex())
