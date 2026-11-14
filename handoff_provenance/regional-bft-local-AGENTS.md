# Regional BFT development

Use [DESIGN.md](DESIGN.md) for protocol rules, implementation mappings, and the comparison contract. Use [README.md](README.md) for the pinned Rust and Commonware environment and the existing smoke scenarios.

## Workflow

1. **Doc.** Freeze a small task contract with the primary source and section, Commonware version, allowed paths, observable behavior, and acceptance cases. The protocol owner decides changes to quorum, signing domains, validity and finality, and recovery. Record unresolved choices before handing the task to a builder.
2. **Implement.** Give each writable path one owner. Use separate Git worktrees for concurrent code changes after establishing a shared Git baseline. Without worktrees, pair one writer with read-only work. The lead owns the common interface, locked dependencies, and comparison settings.
3. **Review and verify.** Have a verifier derive expectations from the source and accepted task contract before reading the implementation. Capture findings and executed checks for the candidate revision. Rerun the affected checks against the integrated revision before using it for experiments.
4. **Simulation.** Run approved scenarios on that verified revision. Save the command, configuration, seeds, traces, and relevant failure output. Distinguish virtual time, local wall time, and AWS measurements.

When the user requests parallel agent work, the main chat coordinates the named specialists. After the contract is fixed, the builder can implement while the verifier prepares independent acceptance cases. Share task IDs and versioned artifacts between agents. Each handoff records the contract, candidate revision or diff, exact checks and outcomes, artifact paths, and open findings.

## Local checks

Use `./scripts/bootstrap.sh` from the project root for formatting and locked tests. Use the README's run commands for specific scenarios. Run one Rust build or heavy simulation at a time on the laptop. Give verification and formal experiment runs separate outputs, and label the current shared-body smoke fixture accordingly.
