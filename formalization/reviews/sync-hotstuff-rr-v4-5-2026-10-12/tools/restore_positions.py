"""Put Protocols 2 and 5 back at their v4.4 source positions (diff copy only)."""
import sys

def _read(path):
    with open(path) as f:
        return f.read()

def _write(path, text):
    with open(path, 'w') as f:
        f.write(text)
src, dst = sys.argv[1:3]
s = _read(src)
def cut(s, begin):
    i = s.index(begin)
    j = s.index('\\end{protocol}', i) + len('\\end{protocol}')
    while j < len(s) and s[j] == '\n':
        j += 1
    return s[:i] + s[j:], s[i:j].rstrip('\n') + '\n\n'
s, p2 = cut(s, '\\begin{protocol}{alg:recovery}')
s, p5 = cut(s, '\\begin{protocol}{alg:force}')
a = 'unrelated recovery.\n\n'
k = s.index(a) + len(a)
s = s[:k] + p2 + s[k:]
b = '\\subsection{Processing rules}\n'
k = s.index(b) + len(b)
s = s[:k] + p5 + s[k:]
_write(dst, s)
