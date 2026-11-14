r"""Tools for the v4.5 revision marks.

strip  IN OUT : replace \Rv{X} by X, drop \Cid{..}, \Chg{X} -> X, and drop the
                \ifrevmarks legend block; gives plain v4.5 source for latexdiff.
black  IN OUT : drop \Rv{X} entirely (keep only black text) for the audit.
"""
import re, sys

def _read(path):
    with open(path) as f:
        return f.read()

def _write(path, text):
    with open(path, 'w') as f:
        f.write(text)

def find_group(s, i):
    """s[i] == '{'; return index just past the matching '}'."""
    depth = 0
    j = i
    while j < len(s):
        c = s[j]
        if c == '\\':
            j += 2
            continue
        if c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return j + 1
        j += 1
    raise ValueError('unbalanced group at %d' % i)

def rewrite(s, mode):
    out = []
    i = 0
    pat = re.compile(r'\\(Rv|Cid|Chg)\{')
    while True:
        m = pat.search(s, i)
        if not m:
            out.append(s[i:])
            break
        # skip the macro definitions in the preamble
        line_start = s.rfind('\n', 0, m.start()) + 1
        if s.startswith('\\newcommand', line_start):
            out.append(s[i:m.end()])
            i = m.end()
            continue
        out.append(s[i:m.start()])
        g0 = m.end() - 1
        g1 = find_group(s, g0)
        inner = s[g0 + 1:g1 - 1]
        name = m.group(1)
        if name == 'Cid':
            pass
        elif name == 'Chg':
            out.append(inner)
        elif mode == 'strip':
            out.append(rewrite(inner, mode))
        else:  # black: drop blue text
            out.append(' ')
        i = g1
    return ''.join(out)

def drop_legend(s):
    a = s.find('\\ifrevmarks\n\\begin{center}')
    if a < 0:
        return s
    b = s.find('\\fi\n', a) + len('\\fi\n')
    return s[:a] + s[b:]

if __name__ == '__main__':
    mode, src, dst = sys.argv[1:4]
    s = _read(src)
    s = drop_legend(s)
    s = rewrite(s, mode)
    if mode == 'strip':
        s = s.replace('\\revmarkstrue', '\\revmarksfalse')
    _write(dst, s)
