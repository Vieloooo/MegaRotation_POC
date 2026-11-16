"use strict";

const data = JSON.parse(document.querySelector("#replay-data").textContent);
const byId = id => document.getElementById(id);
const events = data.events;
const lanes = [
  ["l1", "Anvil L1"], ["espresso", "Espresso"], ["regional", "区域 Final"],
  ["g", "G / GFR"], ["governance", "故障与治理"],
];
const stageLabels = [
  ["outage", "R0 故障"], ["heartbeat", "水位推进"], ["decision", "Signer 判断"],
  ["signed", "达到门槛"], ["prepared", "持久准备"], ["submit", "广播尝试"], ["submitted", "提交确认"], ["include", "L1 收录"], ["finalized", "Finalized"],
  ["reader", "Reader 交付"], ["force", "Force 激活"], ["plan", "历史 Plan"],
  ["new-result", "新区域 Final"], ["accepted", "新块进入 G"],
];
const foreground = new Set(stageLabels.map(([stage]) => stage));
const last = events.length - 1;
const end = events[last].tick_us;
const pct = t => t * 100 / end;
const tick = value => value == null ? "—" : (value / 1_000_000).toFixed(6) + " s";
let cursor = 0;
let timer = null;

function esc(value) {
  const string = value == null ? "—" : typeof value === "object" ? JSON.stringify(value) : String(value);
  return string.replace(/[&<>"']/g, char => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[char]);
}

function facts(rows) {
  return rows.map(([label, value]) => `<div><dt>${esc(label)}</dt><dd><code>${esc(value)}</code></dd></div>`).join("");
}

function observedTip(row) {
  return row ? `q${row.q} · θ ${tick(row.theta_us)}` : "—";
}

function signerFacts(s) {
  return [
    ["结论", s.reason], ["公开 L1 已发布 / 已验证", `${s.l1_advertised ?? "—"} / ${s.l1_verified ?? "—"}`],
    ["Espresso 已发布", observedTip(s.espresso_advertised)],
    ["Espresso 已验证", observedTip(s.espresso_verified)], ["上次观测", observedTip(s.previous)],
    ["q 推进 / θ 推进", `${s.q_advances ? "是" : "否"} / ${s.theta_advances ? "是" : "否"}`],
    ["GFR replay pc", s.replay_position], ["输入就绪", s.ready],
    ["Tail / region / branch / G", `${s.tail} / R${s.region ?? "—"} / β${s.branch} / h${s.head_height}`],
    ["cid", s.cid], ["x_progress", tick(s.progress_us)], ["x_obs", tick(s.observation_us)],
    ["x_obs − x_progress", tick(s.gap_us)], ["门槛 T_stall", tick(s.stall_us)],
    ["时钟可用", s.clock_good], ["时钟检查 τ", tick(s.clock_sample_us)],
    ["独立本地时钟样本", s.clock_sample], ["提议 policy", s.policy],
    ["预计 L1 收录上界", tick(s.predicted_inclusion_us)], ["下一缺失高度过期键", tick(s.expiry_us)],
  ];
}

function inspect(index, showPlan = false) {
  const entry = events[index];
  const padding = (entry.appends || []).map(row => `<span class="${row.type}">h${row.height} · ${row.type}</span>`).join("");
  const rows = entry.signer ? signerFacts(entry.signer) : entry.facts;
  byId("inspector").innerHTML = `<div class="inspector-head"><div><h2>${esc(entry.label)}</h2><p>虚拟 τ ${tick(entry.tick_us)} · ${data.mode === "sample" ? "样本顺序" : "Trace seq"} ${entry.seq}</p></div></div>
    <dl class="facts">${facts(rows)}</dl>${padding ? `<div class="pads">${padding}</div>` : ""}
    ${entry.plan ? `<details ${showPlan ? "open" : ""}><summary>历史 policy 与 Plan</summary><dl class="facts">${facts(entry.plan)}</dl></details>` : ""}
    ${entry.appends ? `<details><summary>虚拟 G 追加逐高证据</summary><div class="table-wrap"><table><thead><tr><th>高度</th><th>类型</th><th>Native hash / state root</th><th>证据</th></tr></thead><tbody>${entry.appends.map(row =>
      `<tr><td>h${row.height}</td><td>${esc(row.type)}</td><td><code>${esc(row.native)}</code><br><code>${esc(row.state_root)}</code></td><td><code>${esc(row.ref)}</code></td></tr>`).join("")}</tbody></table></div></details>` : ""}
    ${entry.refs.length ? `<details open><summary>证据位置</summary><ul>${entry.refs.map(ref => `<li><code>${esc(ref)}</code></li>`).join("")}</ul></details>` : ""}`;
  document.querySelectorAll("[data-frame]").forEach(node => node.setAttribute("aria-current", String(Number(node.dataset.frame) === index)));
  document.querySelectorAll("[data-stage]").forEach(node => node.setAttribute("aria-current", String(entry.stage === node.dataset.stage || (showPlan && node.dataset.stage === "plan"))));
}

function drawTimeline() {
  byId("filters").innerHTML = lanes.map(([name, label]) => `<label><input type="checkbox" data-filter="${name}" checked> ${label}</label>`).join("");
  byId("timeline").classList.add("auto-plot");
  const grid = Array.from({length: Math.floor(end / 5_000_000) + 1}, (_, i) => i * 5_000_000);
  byId("timeline").innerHTML = `<div class="timeline-row" aria-label="刻度"><div class="lane-name">虚拟 τ</div><div class="plot">${grid.map(time => `<span class="ruler-label" style="left:${pct(time)}%">${time / 1_000_000}s</span>`).join("")}</div></div>`
    + lanes.map(([name, label]) => {
      let near = 0;
      let prev = -1_000_000;
      const buttons = events.flatMap((entry, index) => {
        if (!index || entry.lane !== name || (entry.signer && !entry.stage)) return [];
        const marked = foreground.has(entry.stage) || entry.stage === "old-accepted" || entry.stage === "old-result";
        near = entry.tick_us - prev < 330_000 ? near + 1 : 0;
        prev = entry.tick_us;
        const level = [50, 18, 82][near % 3];
        const short = entry.stage === "outage" ? "R0 故障" : entry.stage === "accepted" ? "G 追加"
          : entry.stage === "decision" ? "判定" : entry.stage === "signed" ? "2/3"
          : entry.stage === "force" ? "Force" : entry.label.split(" · ")[0];
        return [`<button type="button" class="mark ${name} ${marked ? "" : "dot"}" aria-label="${esc(entry.label)}" title="${esc(entry.label)}" data-frame="${index}" style="left:${pct(entry.tick_us)}%;--mark-top:${level}%">${marked ? esc(short) : ""}</button>`];
      });
      return `<div class="timeline-row" data-lane="${name}"><div class="lane-name">${label}<span class="lane-head"></span></div><div class="plot"><div class="track"></div><div class="cursor"></div>${buttons.join("")}</div></div>`;
    }).join("");
  byId("stages").innerHTML = stageLabels.flatMap(([stage, label]) => {
    const index = events.findIndex(entry => entry.stage === stage || (stage === "plan" && entry.plan));
    return index < 0 ? [] : [`<button type="button" data-stage="${stage}" data-seek="${index}">${label}</button>`];
  }).join("");
}

function signerCards(seen) {
  const polls = seen.flatMap((entry, index) => entry.signer ? [{...entry, index}] : []);
  const signatures = new Set(seen.filter(entry => entry.signedBy != null).map(entry => entry.signedBy));
  const cards = Array.from({length: data.profile.signer_count}, (_, id) => {
    const observations = polls.filter(entry => entry.signer.signer === id);
    const signed = signatures.has(id);
    const selection = (signed && observations.findLast(entry => entry.signer.eligible)) || observations.at(-1);
    const label = `Signer ${id} · ${signed ? "已签署" : selection?.signer.reason ?? "尚无轮询"}`;
    return selection ? `<button class="signer-card" data-inspect="${selection.index}"><strong>${esc(label)}</strong><span>${esc(observedTip(selection.signer.espresso_verified))}</span><span>停滞差值 ${tick(selection.signer.gap_us)}</span></button>`
      : `<div class="signer-card"><strong>${esc(label)}</strong></div>`;
  });
  byId("signer-cards").innerHTML = cards.join("");
  if (byId("poll-history").open) drawPolls(polls);
}

function drawPolls(polls) {
  byId("polls").innerHTML = `<div class="table-wrap"><table class="poll-table"><thead><tr><th>虚拟 τ</th><th>Signer</th><th>结论</th><th>已验证</th><th>差值</th></tr></thead><tbody>${polls.map(entry =>
    `<tr><td>${tick(entry.tick_us)}</td><td>${entry.signer.signer}</td><td><button data-inspect="${entry.index}">${esc(entry.signer.reason)}</button></td><td>${esc(observedTip(entry.signer.espresso_verified))}</td><td>${tick(entry.signer.gap_us)}</td></tr>`).join("")}</tbody></table></div>`;
}

function seek(index, showPlan = false) {
  cursor = Math.max(0, Math.min(last, index));
  const seen = events.slice(0, cursor + 1);
  byId("clock").textContent = tick(events[cursor].tick_us);
  byId("head").textContent = "h" + seen.findLast(row => row.head != null).head;
  byId("context").textContent = seen.findLast(row => row.duty != null).duty;
  byId("position").textContent = seen.findLast(row => row.pc != null)?.pc ?? "Genesis";
  byId("scrub").value = cursor;
  byId("step-label").textContent = `${cursor} / ${last}`;
  byId("previous").disabled = cursor === 0;
  byId("next").disabled = cursor === last;
  for (const [lane] of lanes) {
    const node = document.querySelector(`[data-lane="${lane}"]`);
    node.querySelector(".cursor").style.left = pct(events[cursor].tick_us) + "%";
    const latest = seen.findLast(entry => entry.lane === lane);
    node.querySelector(".lane-head").textContent = lane === "espresso" ? latest?.water?.split(" ")[0] ?? "—" : "";
    node.querySelectorAll("[data-frame]").forEach(mark => { mark.hidden = Number(mark.dataset.frame) > cursor; });
  }
  const scroll = document.querySelector(".timeline-scroll");
  const line = Array.from(scroll.querySelectorAll(".cursor")).find(node => !node.closest("[hidden]"));
  if (line && scroll.scrollWidth > scroll.clientWidth) {
    const left = line.getBoundingClientRect().left - scroll.getBoundingClientRect().left;
    if (left > scroll.clientWidth - 24 || left < 120) scroll.scrollLeft += left - scroll.clientWidth * .65;
  }
  signerCards(seen);
  inspect(cursor, showPlan);
}

function pause() {
  if (timer !== null) clearTimeout(timer);
  timer = null;
  byId("play").textContent = "▶ 播放";
  byId("play").setAttribute("aria-pressed", "false");
}

function playNext() {
  if (cursor >= last) return pause();
  const gap = (events[cursor + 1].tick_us - events[cursor].tick_us) / 1800;
  timer = setTimeout(() => { seek(cursor + 1); playNext(); }, Math.max(30, Math.min(800, gap)));
}

const mode = byId("mode");
mode.textContent = data.mode === "sample"
  ? "合同样本 · 计划值、推导值与合成诊断记录 · 无真实 L1 收据或验签结论"
  : "正式封存 AF01 · " + data.provenance.candidate + " · seed " + data.profile.seed;
mode.classList.toggle("sample", data.mode === "sample");
byId("source").innerHTML = facts(Object.entries(data.provenance));
drawTimeline();
byId("scrub").max = last;
seek(0);
byId("scrub").addEventListener("input", e => { pause(); seek(Number(e.target.value)); });
byId("reset").addEventListener("click", () => { pause(); seek(0); });
byId("previous").addEventListener("click", () => { pause(); seek(cursor - 1); });
byId("next").addEventListener("click", () => { pause(); seek(cursor + 1); });
byId("play").addEventListener("click", () => {
  if (timer !== null) return pause();
  if (cursor === last) seek(0);
  byId("play").textContent = "Ⅱ 暂停";
  byId("play").setAttribute("aria-pressed", "true");
  playNext();
});
byId("filters").addEventListener("change", e => {
  document.querySelector(`[data-lane="${e.target.dataset.filter}"]`).hidden = !e.target.checked;
});
byId("poll-history").addEventListener("toggle", () => signerCards(events.slice(0, cursor + 1)));
document.addEventListener("click", e => {
  const target = e.target.closest("button");
  if (!target) return;
  if ("seek" in target.dataset) { pause(); seek(Number(target.dataset.seek), target.dataset.stage === "plan"); }
  if ("frame" in target.dataset) { pause(); seek(Number(target.dataset.frame)); }
  if ("inspect" in target.dataset) {
    pause();
    inspect(Number(target.dataset.inspect));
    byId("inspector").scrollIntoView({block: "nearest"});
  }
});
