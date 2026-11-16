function forceStageEvents() {
  const control = data.control;
  const l1 = data.l1[control.position[0]];
  const stage = (ref, name, label, title, level) => ({
    tick: ref.tick_us, seq: ref.seq, stage: name, label, title, level,
  });
  return [
    stage(control.submit, "submit", "提交", "L1 提交 Force 交易", 2),
    stage(l1.included, "include", "收录", "Anvil 收录 Force 交易", 0),
    stage(l1.finalized, "finalize", "final", "ℓ6 模拟 finalized", 0),
    stage(l1.readers.A.trace, "reader-a", "A", "ℓ6 控制前缀交付 A", 1),
    stage(l1.readers.B.trace, "reader-b", "B", "ℓ6 控制前缀交付 B", 2),
  ];
}

function forceL1Events() {
  const events = data.l1.filter((row) => row.readers.A && !row.controls.length).map((row) => ({
    tick: row.readers.A.trace.tick_us, seq: row.readers.A.trace.seq, index: row.height,
    label: `ℓ${row.height}`, title: `L1 reader A 已读 ℓ${row.height}`, level: 1,
  }));
  forceStageEvents().forEach((event, i) => events.push({ ...event, index: data.l1.length + i }));
  return events;
}

function observed(ref) {
  return ref && (ref.tick_us < state.tick || (ref.tick_us === state.tick && ref.seq <= state.seq));
}

function forceL1Details(row) {
  const source = data.control;
  const refs = [
    ["实际 Anvil header", code(`${source.rpc_file} · blocks[${row.height}]`)],
    ["Hub 读入 header", code(`${source.hub_file}:${row.mine_hub_line}`)],
  ];
  for (const [reader, delivery] of Object.entries(row.readers)) {
    if (observed(delivery.trace)) {
      refs.push([`Reader ${reader}`, trace(delivery.trace) + " · " + code(`${source.hub_file}:${delivery.hub_line}`)]);
    }
  }
  const readers = Object.entries(row.readers).filter(([,r]) => observed(r.trace))
    .map(([id,r]) => `${id} ${seconds(r.trace.tick_us)}`);
  const revealed = observed(row.included) || row.height === 0;
  const finalized = observed(row.finalized) ? seconds(row.finalized_us) : row.height === 0 ? "genesis" : "尚未观察";
  const controls = row.controls.length && readers.length
    ? `ForceRecorded · #${source.seq} · (ℓ${source.position.join(", ")})` : "在此位置尚无可读控制";
  return title(`Anvil L1 · ℓ${row.height}`, "实际 L1 header 与 finalized reader 交付", "L1")
    + metricRows([
      ["Header timestamp", formatUs(row.timestamp_us)], ["实际 Anvil hash", code(row.hash)],
      ["实际 Parent hash", code(row.parent)], ["收录 τ", revealed && row.included ? seconds(row.included.tick_us) : "—"],
      ["模拟 finalized τ", finalized], ["已读 τ", readers.join(" · ") || "尚未观察"],
      ["可读 Force", controls],
      ["Worker metadata fixture hash", code(data.worker_l1[row.height].hash)],
    ])
    + evidence(refs);
}

function forceL1Stage(event) {
  const control = data.control;
  const block = data.l1[control.position[0]];
  const stages = forceStageEvents();
  const refs = [control.submit, block.included, block.finalized,
    block.readers.A.trace, block.readers.B.trace];
  const hubLines = [control.submit_hub_line, block.mine_hub_line, block.finalize_hub_line,
    block.readers.A.hub_line, block.readers.B.hub_line];
  const status = stages.map((step, i) => [step.title, observed(refs[i]) ? seconds(step.tick) : "尚未观察"]);
  const evidenceRows = stages.flatMap((step, i) => observed(refs[i]) ? [
    [step.title, trace(refs[i]) + " · " + code(`${control.hub_file}:${hubLines[i]}`)],
  ] : []);
  const facts = [["交易 hash", code(control.tx_hash)], ...status];
  if (observed(block.included)) {
    facts.push(
      ["Anvil header", `ℓ${block.height} · ${code(block.hash)}`],
      ["Parent hash", code(block.parent)],
      ["收录 header timestamp", seconds(block.timestamp_us)],
      ["Receipt", `status ${control.receipt_status} · ForceRecorded #${control.seq} · (${control.position.join(", ")})`],
      ["公开激活键", `${seconds(block.timestamp_us)} + Mₐ ${seconds(data.profile.announcement_us)} = ${seconds(control.act_us)}`],
    );
    evidenceRows.push(
      ["成功回执与 ForceRecorded", code(`${control.rpc_file} · receipts[${control.receipt_index}].logs[${control.receipt_log_index}]`)],
      ["Receipt case", code(`${control.cases_file} · scheduled_healthy_force`)],
      ["Anvil header", code(`${control.rpc_file} · blocks[${block.height}]`)],
      ["独立收录与 L1 reader 报告", code("controls.json[0] · report.json · receipts / stages")],
    );
  }
  return title(event.title, "收录、模拟 finalized 和 reader 交付各有自己的虚拟时刻", "L1 控制")
    + metricRows(facts)
    + evidence(evidenceRows);
}

function forceDetails() {
  const force = data.force;
  const control = data.control;
  const l1 = data.l1[control.position[0]];
  const old = contexts.get(force.old);
  const next = contexts.get(force.new);
  const q = data.espresso[force.q_star];
  const x = force.position.key_us;
  const facts = metricRows([
    ["公开位置", code(`Force(${control.position.join(", ")}) · x = ${seconds(x)}`)],
    ["实际处理 τ", formatUs(force.replay.tick_us)],
    ["已收录 L1", `ℓ${l1.height} · timestamp ${seconds(l1.timestamp_us)}`],
    ["模拟 finalized / reader", `${seconds(l1.finalized_us)} · A ${seconds(l1.readers.A.trace.tick_us)} · B ${seconds(l1.readers.B.trace.tick_us)}`],
    ["当前 G", `h${force.head_height} · 保留全部已接纳区块`],
    ["Lifecycle", code("BranchClosed(FORCE) → SessionAssigned")],
    ["下一职责", `${contextLabel(next)} · ${windowLabel(next)}`],
    ["连接 anchor / Plan reference", `h${force.anchor.height} · ${code(force.anchor.physical)}`],
    ["控制请求的 L1 floor", String(control.l1_floor)],
    ["激活后的 L1 floor", `max(${force.old_floor}, ${l1.height}) = ${force.floor}`],
  ]);
  const rules = metricRows([
    ["覆盖 · §3.4", `x − Mₐ = ${seconds(x - data.profile.announcement_us)}，reader A 的完整控制前缀覆盖至 ℓ${l1.height}。q${q.q} θ ${seconds(q.key_us)}，finalized τ ${seconds(q.finalized_us)} 提供 Espresso watermark。anchor 的 origin ℓ${force.anchor.origin} 也已读。`],
    ["同键全序 · §3.3", `L1 controls → Expire → SessionEnd → Espresso。q* = ${q.q}，激活前评估完 q0–q${q.q - 1}。同键 q${q.q} 的公开内容为 watermark，Espresso 转移排在 Force 之后。`],
    ["职责 · §4.4", `Force 提交时 R0 生产，G 到 h${control.submit_head}。在途 Regular 把 β${old.beta} 的职责交给 R${old.region}。旧 NEXT 未向 G 贡献 physical block，保留 r${old.session} 并递增 β，选择 R${next.region}。`],
    ["关闭 · §4.4", `β${old.beta} 全部分支职责关闭。R0 在满窗时已完成，R${old.region} 的实例执行 Force 关闭。`],
    ["窗口 · §4.4", `x + T_init = ${seconds(x + force.init_us)}，slot = ${force.slot}，h_min = max(${force.anchor.height + 1}, ${force.slot + 1}) = ${force.window[0]}，h_max = h_min + TB − 1 = ${force.window[1]}。`],
  ], "guards");
  return title("Force 激活 · R1 → R2", "同键 q10 已公开，Force 在 GFR 中先完成", "Force")
    + `<div class="explanation"><div><h3>封存记录观察</h3>${facts}</div><div><h3>按规则核对</h3>${rules}</div></div>`
    + `<div class="trace-links">${linkButton("l1-header", l1.height, `查看 ℓ${l1.height} L1 记录`, true)}
       ${linkButton("espresso", q.q, `查看 Espresso q${q.q}`, true)}</div>`
    + evidence([
      ["Force GfrReplayRecord", trace(force.replay)], ["ForceActivated", trace(force.activated)],
      ["独立核对", code("public-replay.json · force[0] · report.json · force.activation")],
      ["新分支 descriptor", code(data.context_files[2])], ["Lifecycle IDs", force.event_ids.map(code).join(" · ")],
      ["Anchor native hash / state root", code(force.anchor_native) + " · " + code(force.anchor_state)],
      ["覆盖与窗口参数", code("raw/config.bin · raw/profile.json")],
      ...force.closes.map((group) => [
        `R${group.region} · ${group.reason}`, group.rows.map(trace).join("<br>"),
      ]),
    ]);
}

function exclusionDetails(entry) {
  const body = entry.body;
  const next = contexts.get(data.force.new);
  return title(`排除旧 R1 physical h${body.height} / q${entry.q}`,
    "q10 完成重放，G 保留 h10", "同键 Espresso")
    + metricRows([
      ["公开时间键 θ", formatUs(entry.position.key_us)], ["实际处理 τ", formatUs(entry.replay.tick_us)],
      ["完整公开", `q_avail = q${entry.q_avail} · 最终证明与正文可核对`],
      ["旧物理区块", code(body.physical)], ["旧 cid", code(entry.cid)],
      ["当前 NEXT", `${contextLabel(next)} · ${windowLabel(next)}`],
      ["G 追加", code("[]")],
    ])
    + metricRows([
      ["排除条件 · §3.3 / §4.2 / §4.4", "β0 已在同一公开键被 Force 关闭，旧 R1 cid 已无职责。"],
      ["后续 q11", "q11 接纳 R2 physical h16 时派生虚拟 padding h11。该 padding 随 h16 的证明进入 G。"],
    ], "guards")
    + `<div class="trace-links">${linkButton("regional", entry.q, "查看旧 R1 区块", true)}
       ${linkButton("espresso", entry.q, "查看公开 q10", true)}</div>`
    + evidence([
      ["延迟发布", trace(entry.held)], ["PublisherRecorded", trace(entry.publisher)], ["空转移", trace(entry.replay)],
      ["LatePublicationExcluded", trace(entry.excluded)], ["区域结果", code(entry.result_file)],
      ...proofEvidence(entry),
    ]);
}

function paddingDetails(entry) {
  if (!entry.padding?.length) return "";
  return `<details open><summary>q${entry.q} 一次追加 h${entry.append[0]}–h${entry.append[1]}</summary>
    <p class="muted">h11–h15 是虚拟 padding。证明来源为 R2 physical h16。</p>
    <div class="table-wrap"><table>
    <thead><tr><th>高度</th><th>类型</th><th>Native hash / state root</th><th>证据</th></tr></thead>
    <tbody>${entry.padding.map((row) => `<tr><td>h${row.height}</td><td>padding</td>
      <td>${code(row.native)}<br>${code(row.state_root)}</td>
      <td>${trace(row.trace)}<br>${code(`public-replay.json · expected[${row.expected_index}]`)}</td></tr>`).join("")}
    <tr><td>h${entry.body.height}</td><td>physical</td><td>${code(entry.native)}<br>${code(entry.state_root)}</td>
      <td>${trace(entry.accept)}<br>${code(`public-replay.json · expected[${entry.oracle.expected_index}]`)}</td></tr>
    </tbody></table></div></details>`;
}
