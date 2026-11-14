"""Build the self-contained illustrated reading edition from the Markdown note."""
from pathlib import Path
import re
import subprocess

assets = Path(__file__).resolve().parent
learning = assets.parent.parent
source = learning / 'zh-CN' / '16_simplex_and_regional_bft.md'
target = source.with_suffix('.html')
subprocess.run([
    'pandoc', source.name, '--standalone', '--mathml', '--embed-resources',
    '--toc', '--toc-depth=2', '--metadata', 'lang=zh-CN',
    '--metadata', 'pagetitle=Simplex 中文学习笔记与区域 BFT 选型',
    '--css=../figures/simplex/reading.css', '-o', target.name,
], cwd=source.parent, check=True)
html = target.read_text()
toc = re.search(r'<nav id="TOC".*?</nav>', html, re.S)
if toc:
    block = toc.group(0)
    html = html[:toc.start()] + html[toc.end():]
    first_section = html.index('<h2 ')
    html = html[:first_section] + block + '\n' + html[first_section:]
target.write_text(html)
print(target)
