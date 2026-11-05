# usage: python gen.py <repo> <out_base>  -> <out_base>.md / .csv
import yaml,glob,csv,sys,subprocess,io
repo,out=sys.argv[1],sys.argv[2]
prog=yaml.safe_load(open(f'{repo}/content/program.yaml'))['courses']
idx={l['id']:l for l in yaml.safe_load(open(f'{repo}/content/index.yaml'))['lectures']}
sha=subprocess.run(['git','-C',repo,'rev-parse','--short','HEAD'],capture_output=True,text=True).stdout.strip()
rows=[];per=[]
for c in prog:
    items=[]
    for f in sorted(glob.glob(f"{repo}/content/{c['id']}/L*.yaml")):
        doc=yaml.safe_load(open(f).read().replace('?','\u2047'))
        for it in doc['items']:
            if it.get('kind')=='enumerate' and it.get('hook'):
                it['_lec']=doc.get('title'); items.append(it)
    if items: per.append((c,items))
total=sum(len(i) for _,i in per)
md=["# تحشيشات قوايم «اذكر/عدّد» في Batna","",
 f"الملف الوحيد لكل التحشيشات، متولّد من محتوى التطبيق على main (commit {sha}). لكل قايمة: السؤال، والنقط بالإنجليزي اللي بتتحفظ، وصفحة الكتاب مع الجملة اللي منه، والتحشيشة.","",
 "| المادة | الكورس | القوايم |","|---|---|---|"]
for c,i in per: md.append(f"| {c['short']} ({c['id']}) | {c['title']} | {len(i)} |")
md+= [f"| **المجموع** | | **{total}** |",""]
buf=io.StringIO(); w=csv.writer(buf)
w.writerow(['course','id','lecture','question','points','pass','book_page','book_quote','tahshisha'])
for c,items in per:
    md+=[f"## {c['short']}: {c['title']} ({c['id']}), {len(items)} قايمة",""]
    for it in items:
        lec=it['_lec'] or idx[it['id'].rsplit('.',1)[0]]['title']; src=it.get('src') or {}
        md+=[f"### {it['id']} · {lec}","",f"**Q:** {it['prompt']}",""]
        md+=[f"{n}. {p}" for n,p in enumerate(it['points'],1)]+[""]
        if src.get('page') is not None:
            md+=[f"*Book p. {src['page']}*"+(f" — \"{src['quote']}\"" if src.get('quote') else ""),""]
        md+=[f"> 😄 {it['hook']}",""]
        w.writerow([c['id'],it['id'],lec,it['prompt'],' | '.join(it['points']),it['pass'],src.get('page',''),src.get('quote',''),it['hook']])
open(out+'.md','w').write('\n'.join(md).replace('\u2047','?'))
open(out+'.csv','w',encoding='utf-8-sig',newline='').write(buf.getvalue().replace('\u2047','?'))
print(total,[(c['id'],len(i)) for c,i in per])
