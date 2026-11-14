from pathlib import Path
from xml.etree.ElementTree import Element, SubElement, ElementTree
from math import atan2, sin, cos
import re
from html import escape
from reportlab.pdfgen import canvas
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.lib.colors import HexColor

OUT = Path(__file__).resolve().parents[1]
pdfmetrics.registerFont(TTFont('Arial', '/System/Library/Fonts/Supplemental/Arial.ttf'))
pdfmetrics.registerFont(TTFont('ArialBold', '/System/Library/Fonts/Supplemental/Arial Bold.ttf'))
PALETTE = {
    'blue': ('#EDF4FA', '#426A8C'),
    'green': ('#EFF6F0', '#507859'),
    'amber': ('#FCF3E5', '#9B7B45'),
    'gray': ('#F5F6F7', '#7B8389'),
}

def box(x,y,w,h,lines,color='blue',size=14.5):
    return dict(x=x,y=y,w=w,h=h,lines=lines,color=color,size=size)

def label(x,y,w,lines,size=13.5,bold=False):
    return dict(x=x,y=y,w=w,h=18*len(lines),lines=lines,color=None,size=size,bold=bold)

def edge(points, dashed=False):
    return dict(points=points,dashed=dashed)

def runs(line):
    result=[];last=0
    for m in re.finditer(r'_\(([^)]+)\)|_([A-Za-z])',line):
        result.append((line[last:m.start()],False))
        result.append((m.group(1) or m.group(2),True));last=m.end()
    result.append((line[last:],False));return result

def html_line(line):
    return ''.join('<sub>'+escape(s)+'</sub>' if sub else escape(s) for s,sub in runs(line))

def build(name,width,height,nodes,edges):
    model=Element('mxGraphModel',dict(dx=str(width),dy=str(height),grid='0',page='0',pageScale='1',pageWidth=str(width),pageHeight=str(height),math='0',shadow='0',background='#FFFFFF'))
    root=SubElement(model,'root')
    SubElement(root,'mxCell',id='0')
    SubElement(root,'mxCell',id='1',parent='0')
    c=canvas.Canvas(str(OUT/(name+'.pdf')), pagesize=(width,height))
    c.setTitle({'pipeline':'Ordering, finality, and publication pipeline','local-recovery':'Regional synchronous broadcast and safe completion','global-handoff':'Finalized geographic handoff'}[name])
    c.setAuthor('MegaETH sequencer rotation research draft')
    # Edges render behind all boxes and labels.
    for i,e in enumerate(edges):
        pts=e['points']; dashed=e['dashed']
        style='edgeStyle=none;rounded=0;html=1;strokeColor=#59646E;strokeWidth=1.5;endArrow=block;endFill=1;'
        if dashed: style+='dashed=1;dashPattern=5 4;'
        cell=SubElement(root,'mxCell',id=f'e{i}',value='',style=style,edge='1',parent='1')
        geo=SubElement(cell,'mxGeometry',relative='1',attrib={'as':'geometry'})
        SubElement(geo,'mxPoint',x=str(pts[0][0]),y=str(pts[0][1]),attrib={'as':'sourcePoint'})
        SubElement(geo,'mxPoint',x=str(pts[-1][0]),y=str(pts[-1][1]),attrib={'as':'targetPoint'})
        if len(pts)>2:
            arr=SubElement(geo,'Array',attrib={'as':'points'})
            for x,y in pts[1:-1]: SubElement(arr,'mxPoint',x=str(x),y=str(y))
        c.saveState();c.setStrokeColor(HexColor('#59646E'));c.setFillColor(HexColor('#59646E'));c.setLineWidth(1.5)
        if dashed: c.setDash(5,4)
        p=c.beginPath();p.moveTo(pts[0][0],height-pts[0][1])
        for x,y in pts[1:]: p.lineTo(x,height-y)
        c.drawPath(p)
        x,y=pts[-1];px,py=pts[-2];ang=atan2(y-py,x-px)
        tip=c.beginPath();tip.moveTo(x,height-y)
        for sign in [-1,1]:
            xx=x-7*cos(ang)+sign*3.5*sin(ang); yy=y-7*sin(ang)-sign*3.5*cos(ang)
            tip.lineTo(xx,height-yy)
        tip.close();c.setDash();c.drawPath(tip,stroke=0,fill=1);c.restoreState()
    for i,n in enumerate(nodes):
        x,y,w,h=n['x'],n['y'],n['w'],n['h'];size=n['size'];lines=n['lines'];colored=n['color'] is not None
        fill,stroke=PALETTE[n['color']] if colored else ('none','none')
        style=f'rounded=1;arcSize=8;whiteSpace=wrap;html=1;align=center;verticalAlign=middle;fontFamily=Arial;fontSize={size};fontColor=#182633;fillColor={fill};strokeColor={stroke};strokeWidth=1.3;spacing=8;'
        if n.get('bold'):style+='fontStyle=1;'
        cell=SubElement(root,'mxCell',id=f'n{i}',value='<br>'.join(map(html_line,lines)),style=style,vertex='1',parent='1')
        SubElement(cell,'mxGeometry',x=str(x),y=str(y),width=str(w),height=str(h),attrib={'as':'geometry'})
        if colored:
            c.setFillColor(HexColor(fill));c.setStrokeColor(HexColor(stroke));c.setLineWidth(1.3)
            c.roundRect(x,height-y-h,w,h,6,fill=1,stroke=1)
        c.setFillColor(HexColor('#182633'));c.setFont('ArialBold' if n.get('bold') else 'Arial',size)
        lineh=size*1.27; top=y+(h-lenh(lines,lineh))/2+size*.81
        for j,line in enumerate(lines):
            font='ArialBold' if n.get('bold') else 'Arial'
            spans=runs(line)
            lengths=[pdfmetrics.stringWidth(s,font,size*.7 if sub else size) for s,sub in spans]
            assert sum(lengths)<=w-10, (name,line)
            cursor=x+(w-sum(lengths))/2
            for (s,sub),spanw in zip(spans,lengths):
                c.setFont(font,size*.7 if sub else size)
                c.drawString(cursor,height-(top+j*lineh)-(size*.2 if sub else 0),s)
                cursor+=spanw
    ElementTree(model).write(OUT/(name+'.drawio'),encoding='utf-8',xml_declaration=True)
    c.showPage();c.save()

def lenh(lines,lineh):return len(lines)*lineh

build('pipeline',620,318,[
    label(8,3,604,['REGIONAL ORDERING'],bold=True),
    box(8,32,137,80,['S streams signed','order packets']),
    box(164,32,137,80,['Local LMD-GHOST','speculative head']),
    box(320,32,137,80,['2δ-BB checkpoint','up to B_C packets']),
    box(476,32,137,80,['A2 export: RC_u','>2/3 DECIDED']),
    label(9,145,603,['CROSS-REGION FFG AND PUBLICATION'],bold=True),
    box(8,174,137,88,['Global candidate','complete data','+ availability AC'],'green'),
    box(164,174,137,88,['FFG justified','A3: provisional'],'green'),
    box(320,174,137,88,['FFG finalized','A4: stable'],'green'),
    box(476,174,137,88,['P job → native','output → DA / L1','A5: settled'],'amber'),
    label(9,284,603,['P may execute A2 inputs speculatively; irreversible jobs require A4.']),
],[
    edge([(145,72),(164,72)]),edge([(301,72),(320,72)]),edge([(457,72),(476,72)]),
    edge([(544,112),(544,130),(76,130),(76,174)]),
    edge([(145,218),(164,218)]),edge([(301,218),(320,218)]),edge([(457,218),(476,218)]),
])

build('local-recovery',620,350,[
    label(8,3,604,['GOOD CASE: INTERNAL DECISION AT 2δ; EXPORT ADDS LATENCY'],bold=True),
    box(8,33,137,108,['Leader broadcasts','checkpoint x_u','with full data']),
    box(164,33,137,108,['All-to-all VOTE','At most one','value per member']),
    box(320,33,137,108,['>2/3 vote QC','Forward; set lock','Decide if received','by 2Δ + σ']),
    box(476,33,137,108,['After decision:','>2/3 DECIDED','exports RC_u','to P / global']),
    label(8,172,604,['MANDATORY COMPLETION: EARLY DECIDERS ALSO PARTICIPATE'],bold=True),
    box(8,207,186,78,['At 3Δ + 2σ','Everyone: QC or not','BA(lock snapshot)'],'gray'),
    box(217,207,186,78,['Undecided decide / export','All continue helping','until BA completes'],'gray'),
    box(426,207,187,78,['After BA + export:','next instance / leader','inherits resolved prefix']),
    label(8,311,604,['Late vote QCs cannot prove a fast decision; exporting adds relay latency.']),
],[
    edge([(145,87),(164,87)]),edge([(301,87),(320,87)]),edge([(457,87),(476,87)]),
    edge([(388,141),(388,157),(4,157),(4,246),(8,246)]),
    edge([(194,246),(217,246)]),edge([(403,246),(426,246)]),
    edge([(575,141),(575,207)]),
])

build('global-handoff',620,391,[
    label(8,3,604,['EXAMPLE: TWO-HOLD HEALTH GUARD'],bold=True),
    box(8,32,186,90,['C_(e−1): anchor','generation a','input tip q'],'green'),
    box(217,32,186,90,['C_e: HOLD','same input tip q','new control hash'],'green'),
    box(426,32,187,90,['C_(e+1): HOLD','same input tip q','link finalizes C_e'],'green'),
    label(8,143,604,['READY successor retrieves the complete inherited state and input data.']),
    box(8,196,186,94,['C_(e+2): ROTATE','pending generation a+1','same input tip q']),
    box(217,196,186,94,['C_(e+3): CONFIRM','same pending state','link finalizes ROTATE']),
    box(426,196,187,94,['Finality proof π_R','authorizes successor;','later record consumes it']),
    label(8,317,604,['Pending descendants do not advance data; only finalized ROTATE activates.']),
    label(8,345,604,['Preserve A4; cancel unretained A2/A3 tails and fence fresh old-generation output.'],size=13),
],[
    edge([(194,77),(217,77)]),edge([(403,77),(426,77)]),
    edge([(519,122),(519,135),(4,135),(4,243),(8,243)]),
    edge([(194,243),(217,243)]),edge([(403,243),(426,243)]),
])

print('Wrote three editable draw.io sources and three matching vector PDFs.')
