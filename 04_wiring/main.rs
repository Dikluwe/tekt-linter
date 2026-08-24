//! Crystalline Lineage
//! @prompt 00_nucleo/prompts/linter-core.md
//! @prompt-hash 80859d12
//! @layer L4
//! @updated 2026-08-24

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser as ClapParser;
use rayon::prelude::*;

use crystalline_lint::contracts::citation_freshness::{
    CitationFreshnessResolver, UnknownCitationFreshness,
};
use crystalline_lint::contracts::file_provider::{FileProvider, SourceError, SourceFile};
use crystalline_lint::contracts::language_parser::LanguageParser;
use crystalline_lint::contracts::parse_error::ParseError;
use crystalline_lint::contracts::prompt_provider::PromptProvider;
use crystalline_lint::contracts::prompt_reader::PromptReader;
use crystalline_lint::contracts::prompt_snapshot_reader::PromptSnapshotReader;
use crystalline_lint::entities::l1_allowed_external::{L1AllowedExternal, L1AllowedExternalSet};
use crystalline_lint::entities::layer::Language;
use crystalline_lint::entities::parsed_file::{ParsedFile, PublicInterface, WiringConfig};
use crystalline_lint::entities::project_index::{LocalIndex, ProjectIndex};
use crystalline_lint::entities::refinement::{
    compare_refinement, ArtifactFacts, ObservableValue, UnknownReason,
};
use crystalline_lint::entities::refinement_seal::{accepts, OracleKind, VerdictName};
use crystalline_lint::entities::violation::{Location, Violation, ViolationLevel};
use crystalline_lint::infra::c_parser::CParser;
use crystalline_lint::infra::citation_freshness::FsCitationFreshnessResolver;
use crystalline_lint::infra::config::CrystallineConfig;
use crystalline_lint::infra::cpp_parser::CppParser;
use crystalline_lint::infra::crate_registry::CrateRegistry;
use crystalline_lint::infra::elixir_parser::ElixirParser;
use crystalline_lint::infra::git_refinement::{
    extract_revision_snapshot, require_self_contained_object_database, resolve_commit,
};
use crystalline_lint::infra::go_parser::GoParser;
use crystalline_lint::infra::hash_writer;
use crystalline_lint::infra::java_parser::JavaParser;
use crystalline_lint::infra::prompt_reader::FsPromptReader;
use crystalline_lint::infra::prompt_snapshot_reader::FsPromptSnapshotReader;
use crystalline_lint::infra::prompt_walker::FsPromptWalker;
use crystalline_lint::infra::py_parser::PyParser;
use crystalline_lint::infra::refinement_extractor::{
    extract_snapshot, load_observable_specs, load_observable_specs_from_bytes, serialize_snapshot,
    write_snapshot_atomic,
};
use crystalline_lint::infra::refinement_seal::{
    confined_read, load_manifest, read_manifest, receipt, semantic_manifest_sha256, serialize_seal,
    sha256, write_atomic, Seal, SealCounts,
};
use crystalline_lint::infra::refinement_snapshot::{
    load_contract, load_contract_from_bytes, load_snapshot,
};
use crystalline_lint::infra::rs_parser::RustParser;
use crystalline_lint::infra::snapshot_writer;
use crystalline_lint::infra::ts_parser::TsParser;
use crystalline_lint::infra::walker::FileWalker;
use crystalline_lint::infra::zig_parser::ZigParser;
use crystalline_lint::rules::pub_leak::L1Ports;
use crystalline_lint::rules::{
    alien_file, compound_guard, context_erasure, dangling_contract, decision_ownership,
    deep_pattern_nesting, external_type_in_contract, forbidden_import, impure_core,
    multi_prompt_header, mutable_state_core, or_pattern_alternatives, orphan_prompt, prompt_drift,
    prompt_header, prompt_stale, provenance_inventory, pub_leak, quarantine_leak, range_pattern,
    semantic_field_loss, test_file, unsourced_constant, wildcard_saturation, wiring_logic_leak,
};
use crystalline_lint::shell::cli::{validate_args, Cli, EnabledChecks, OutputFormat};
use crystalline_lint::shell::fix_hashes::{self, HashRewriter};
use crystalline_lint::shell::refinement::{
    self, Command as RefinementCommand, RefinementOutputFormat,
};
use crystalline_lint::shell::update_snapshot::{self, SnapshotRewriter};
use std::collections::HashMap;

fn main() {
    let cli = Cli::parse();

    if let Some(RefinementCommand::SealRefinement(args)) = &cli.command {
        let manifest_bytes = read_manifest(&args.manifest).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: refinement seal manifest error: {error}");
            process::exit(2)
        });
        let manifest = load_manifest(&manifest_bytes).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: refinement seal manifest error: {error}");
            process::exit(2)
        });
        let manifest_hash = semantic_manifest_sha256(&manifest).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: refinement seal manifest error: {error}");
            process::exit(2)
        });
        require_self_contained_object_database(&args.repository).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: refinement seal repository error: {error}");
            process::exit(2)
        });
        let prompt_bytes =
            confined_read(&args.repository, &manifest.prompt).unwrap_or_else(|error| {
                eprintln!("crystalline-lint: refinement seal prompt error: {error}");
                process::exit(2)
            });
        if sha256(&prompt_bytes) != manifest.prompt_sha256 {
            eprintln!("crystalline-lint: refinement seal prompt hash mismatch");
            process::exit(2);
        }
        let contract_bytes =
            confined_read(&args.repository, &manifest.contract).unwrap_or_else(|error| {
                eprintln!("crystalline-lint: refinement seal contract error: {error}");
                process::exit(2)
            });
        if sha256(&contract_bytes) != manifest.contract_sha256 {
            eprintln!("crystalline-lint: refinement seal contract hash mismatch");
            process::exit(2);
        }
        let baseline_oid =
            resolve_commit(&args.repository, &manifest.baseline_oid).unwrap_or_else(|error| {
                eprintln!("crystalline-lint: refinement seal baseline error: {error}");
                process::exit(2)
            });
        if manifest.baseline_oid != baseline_oid {
            eprintln!("crystalline-lint: baseline_oid must be the complete canonical commit OID");
            process::exit(2);
        }
        let contract_source = manifest.contract.display().to_string();
        let specs = load_observable_specs_from_bytes(&contract_bytes, &contract_source)
            .unwrap_or_else(|error| {
                eprintln!("crystalline-lint: refinement seal contract error: {error}");
                process::exit(2)
            });
        let contract =
            load_contract_from_bytes(&contract_bytes, &contract_source).unwrap_or_else(|error| {
                eprintln!("crystalline-lint: refinement seal contract error: {error}");
                process::exit(2)
            });
        let mut receipts = Vec::with_capacity(manifest.oracles.len());
        for oracle in &manifest.oracles {
            let before_oid =
                resolve_commit(&args.repository, &oracle.before_ref).unwrap_or_else(|error| {
                    eprintln!(
                        "crystalline-lint: oracle `{}` before ref error: {error}",
                        oracle.id
                    );
                    process::exit(2)
                });
            let after_oid =
                resolve_commit(&args.repository, &oracle.after_ref).unwrap_or_else(|error| {
                    eprintln!(
                        "crystalline-lint: oracle `{}` after ref error: {error}",
                        oracle.id
                    );
                    process::exit(2)
                });
            let before = extract_revision_snapshot(&args.repository, &before_oid, &specs)
                .unwrap_or_else(|error| {
                    eprintln!(
                        "crystalline-lint: oracle `{}` before extraction error: {error}",
                        oracle.id
                    );
                    process::exit(2)
                });
            let after = extract_revision_snapshot(&args.repository, &after_oid, &specs)
                .unwrap_or_else(|error| {
                    eprintln!(
                        "crystalline-lint: oracle `{}` after extraction error: {error}",
                        oracle.id
                    );
                    process::exit(2)
                });
            let exhausted = |facts: &ArtifactFacts| {
                facts.observables.values().any(|value| {
                    matches!(
                        value,
                        ObservableValue::Unknown(UnknownReason::BudgetExhausted)
                    )
                })
            };
            if exhausted(&before) || exhausted(&after) {
                eprintln!(
                    "crystalline-lint: oracle `{}` exhausted the Git input budget",
                    oracle.id
                );
                process::exit(2);
            }
            let verdict = compare_refinement(&contract, &before, &after);
            let verdict_name = VerdictName::from(&verdict);
            if !accepts(oracle.kind, &verdict) {
                eprintln!(
                    "crystalline-lint: oracle `{}` expected {} but returned {}",
                    oracle.id,
                    oracle.kind.as_str(),
                    verdict_name.as_str()
                );
                process::exit(2);
            }
            receipts.push(receipt(
                oracle.id.clone(),
                oracle.kind,
                before_oid,
                after_oid,
                verdict_name,
            ));
        }
        receipts.sort_by(|a, b| a.id.cmp(&b.id));
        let positive_count = manifest
            .oracles
            .iter()
            .filter(|o| o.kind == OracleKind::Positive)
            .count();
        let negative_count = manifest
            .oracles
            .iter()
            .filter(|o| o.kind == OracleKind::Negative)
            .count();
        let unknown_count = manifest
            .oracles
            .iter()
            .filter(|o| o.kind == OracleKind::Unknown)
            .count();
        let seal = Seal {
            protocol_version: manifest.protocol_version,
            manifest_sha256: &manifest_hash,
            prompt_sha256: &manifest.prompt_sha256,
            contract_sha256: &manifest.contract_sha256,
            baseline_oid: &baseline_oid,
            contract_producer: &manifest.contract_producer,
            implementation_producer: &manifest.implementation_producer,
            verifier_producer: &manifest.verifier_producer,
            receipts: &receipts,
            counts: SealCounts {
                positive: positive_count,
                negative: negative_count,
                unknown: unknown_count,
            },
            mutation_score: "1.0",
            sealed: true,
        };
        let bytes = serialize_seal(&seal).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: {error}");
            process::exit(2)
        });
        write_atomic(&args.output, &bytes).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: {error}");
            process::exit(2)
        });
        process::exit(0);
    }

    if let Some(RefinementCommand::Refine(args)) = &cli.command {
        let source = load_snapshot(&args.before).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: refinement input error: {error}");
            process::exit(2);
        });
        let target = load_snapshot(&args.after).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: refinement input error: {error}");
            process::exit(2);
        });
        let contract = load_contract(&args.contract).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: refinement contract error: {error}");
            process::exit(2);
        });
        let verdict = compare_refinement(&contract, &source, &target);
        match args.format {
            RefinementOutputFormat::Text => print!("{}", refinement::format_text(&verdict)),
            RefinementOutputFormat::Sarif => println!("{}", refinement::format_sarif(&verdict)),
        }
        process::exit(refinement::exit_code(&verdict));
    }

    if let Some(RefinementCommand::Snapshot(args)) = &cli.command {
        let specs = load_observable_specs(&args.contract).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: snapshot contract error: {error}");
            process::exit(2);
        });
        let snapshot =
            extract_snapshot(&args.path, &args.artifact_id, &specs).unwrap_or_else(|error| {
                eprintln!("crystalline-lint: snapshot extraction error: {error}");
                process::exit(2);
            });
        let content = serialize_snapshot(&snapshot).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: snapshot serialization error: {error}");
            process::exit(2);
        });
        write_snapshot_atomic(&args.output, &content).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: snapshot write error: {error}");
            process::exit(2);
        });
        print!(
            "{}",
            refinement::format_snapshot_success(&args.output, snapshot.observables.len())
        );
        process::exit(0);
    }

    if let Some(RefinementCommand::RefineRevisions(args)) = &cli.command {
        require_self_contained_object_database(&args.repository).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: refinement repository error: {error}");
            process::exit(2)
        });
        let specs = load_observable_specs(&args.contract).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: refinement contract error: {error}");
            process::exit(2);
        });
        let contract = load_contract(&args.contract).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: refinement contract error: {error}");
            process::exit(2);
        });
        let before_oid =
            resolve_commit(&args.repository, &args.before_ref).unwrap_or_else(|error| {
                eprintln!("crystalline-lint: before ref error: {error}");
                process::exit(2);
            });
        let after_oid = resolve_commit(&args.repository, &args.after_ref).unwrap_or_else(|error| {
            eprintln!("crystalline-lint: after ref error: {error}");
            process::exit(2);
        });
        let source = extract_revision_snapshot(&args.repository, &before_oid, &specs)
            .unwrap_or_else(|error| {
                eprintln!("crystalline-lint: before revision extraction error: {error}");
                process::exit(2);
            });
        let target = extract_revision_snapshot(&args.repository, &after_oid, &specs)
            .unwrap_or_else(|error| {
                eprintln!("crystalline-lint: after revision extraction error: {error}");
                process::exit(2);
            });
        eprintln!(
            "crystalline-lint: comparing immutable Git objects {before_oid} -> {after_oid}; working tree ignored"
        );
        let verdict = compare_refinement(&contract, &source, &target);
        match args.format {
            RefinementOutputFormat::Text => print!("{}", refinement::format_text(&verdict)),
            RefinementOutputFormat::Sarif => println!("{}", refinement::format_sarif(&verdict)),
        }
        process::exit(refinement::exit_code(&verdict));
    }

    // ── Arg validation ────────────────────────────────────────────────────────
    if let Err(e) = validate_args(&cli) {
        eprintln!("crystalline-lint: {e}");
        process::exit(1);
    }

    // ── Config ────────────────────────────────────────────────────────────────
    let config_path = if cli.config.is_relative() {
        cli.path.join(&cli.config)
    } else {
        cli.config.clone()
    };
    let config = if config_path.exists() {
        CrystallineConfig::load(&config_path).unwrap_or_else(|e| {
            eprintln!("crystalline-lint: config error: {e}");
            process::exit(1);
        })
    } else {
        CrystallineConfig::default()
    };

    // ── Phase 0: scan all prompts (sequential, before pipeline) ───────────────
    let nucleo_root = cli.path.clone();
    let enabled = EnabledChecks::from_cli(&cli.checks, cli.no_drift, cli.no_stale);

    let orphan_exceptions: std::collections::HashSet<String> =
        config.orphan_exceptions.keys().cloned().collect();
    let prompt_walker = FsPromptWalker::new(cli.path.clone(), orphan_exceptions);
    // Scan prompts only when V7 is enabled — avoids I/O when check is suppressed.
    // --emit-resolution não usa prompts e roda em workspaces arbitrários (sem
    // 00_nucleo): pula o scan para não abortar (instrumentação 0058).
    let all_prompts = if enabled.v7 && !cli.emit_resolution {
        match prompt_walker.scan() {
            Ok(ap) => Some(ap),
            Err(e) => {
                eprintln!("crystalline-lint: prompt scan error: {e:?}");
                process::exit(1);
            }
        }
    } else {
        None
    };

    // ── L1Ports from config ───────────────────────────────────────────────────
    let mut port_set: std::collections::HashSet<String> = config.l1_ports.keys().cloned().collect();
    for v in config.l1_ports.values() {
        port_set.insert(v.clone());
    }
    let l1_ports = L1Ports::new(port_set);

    // ── WiringConfig for V12 ──────────────────────────────────────────────────
    let v21_config = config.v21_rule_config();
    let citation_freshness = FsCitationFreshnessResolver::new(cli.path.clone(), 4 * 1024 * 1024);
    let semantic_levels = [
        config.level_for("V23", ViolationLevel::Warning),
        config.level_for("V24", ViolationLevel::Warning),
        config.level_for("V25", ViolationLevel::Warning),
    ];

    let wiring_config = WiringConfig {
        allow_adapter_structs: config
            .wiring_exceptions
            .allow_adapter_structs
            .unwrap_or(true),
    };

    // ── L1AllowedExternalSet for V14 (por linguagem) ────────────────────────
    let l1_allowed = L1AllowedExternalSet {
        rust: L1AllowedExternal::for_rust(config.l1_allowed_for_language("rust")),
        python: L1AllowedExternal::for_python(config.l1_allowed_for_language("python")),
        typescript: L1AllowedExternal::for_typescript(config.l1_allowed_for_language("typescript")),
        c: L1AllowedExternal::for_c(config.l1_allowed_for_language("c")),
        cpp: L1AllowedExternal::for_cpp(config.l1_allowed_for_language("cpp")),
        zig: L1AllowedExternal::for_zig(config.l1_allowed_for_language("zig")),
        go: L1AllowedExternal::for_go(config.l1_allowed_for_language("go")),
        java: L1AllowedExternal::for_java(config.l1_allowed_for_language("java")),
        elixir: L1AllowedExternal::for_elixir(config.l1_allowed_for_language("elixir")),
    };

    // ── Crate registry (membro→camada + deps) para classificação ciente (0052) ─
    // Construído uma vez do workspace-alvo; vazio se não for projeto cargo (legado).
    let crate_registry = CrateRegistry::from_root(&cli.path, &config).unwrap_or_else(|error| {
        eprintln!("crystalline-lint: crate registry infrastructure error: {error}");
        process::exit(1)
    });

    // ── Instantiate L3 components ─────────────────────────────────────────────
    let shared_prompt_reader = std::sync::Arc::new(
        crystalline_lint::infra::prompt_reader::CachedPromptReader::new(FsPromptReader {
            nucleo_root: nucleo_root.clone(),
        }),
    );
    let shared_snapshot_reader = FsPromptSnapshotReader {
        nucleo_root: nucleo_root.clone(),
    };

    let parser = MultiParser {
        rust: RustParser::new(
            shared_prompt_reader.clone(),
            shared_snapshot_reader.clone(),
            config.clone(),
            crate_registry.clone(),
        ),
        ts: TsParser::new(
            shared_prompt_reader.clone(),
            shared_snapshot_reader.clone(),
            config.clone(),
            cli.path.clone(),
        ),
        py: PyParser::new(
            shared_prompt_reader.clone(),
            shared_snapshot_reader.clone(),
            config.clone(),
            cli.path.clone(),
        ),
        c: CParser::new(
            shared_prompt_reader.clone(),
            shared_snapshot_reader.clone(),
            config.clone(),
            cli.path.clone(),
        ),
        cpp: CppParser::new(
            shared_prompt_reader.clone(),
            shared_snapshot_reader.clone(),
            config.clone(),
            cli.path.clone(),
        ),
        zig: ZigParser::new(
            shared_prompt_reader.clone(),
            shared_snapshot_reader.clone(),
            config.clone(),
            cli.path.clone(),
        ),
        go: GoParser::new(
            shared_prompt_reader.clone(),
            shared_snapshot_reader.clone(),
            config.clone(),
            cli.path.clone(),
        ),
        java: JavaParser::new(
            shared_prompt_reader.clone(),
            shared_snapshot_reader.clone(),
            config.clone(),
            cli.path.clone(),
        ),
        elixir: ElixirParser::new(
            shared_prompt_reader.clone(),
            shared_snapshot_reader.clone(),
            config.clone(),
            cli.path.clone(),
        ),
    };
    let walker = FileWalker::new(cli.path.clone(), config.clone());

    // Collect source files and errors separately so ParsedFile<'a> can borrow
    // from source_files (zero-copy, ADR-0004). all_prompts is declared above
    // source_files so it outlives them — required for V7's check_orphans.
    let (source_files, source_errors) = collect_walker_results(walker.files());

    let (mut all_violations, all_parsed, project_index) = run_pipeline(
        &source_files,
        &source_errors,
        &parser,
        &enabled,
        &l1_ports,
        &wiring_config,
        &l1_allowed,
        &config.analysis.lineage.strict_directories,
        config.check_test_imports.unwrap_or(false),
        &config.wildcard_exceptions,
        &v21_config,
        &semantic_levels,
        &citation_freshness,
    );

    // ── --emit-resolution (instrumentação 0058, fora do selo de veredito) ──────
    // Despeja a resolução de todo import (incl. Unknown) e sai — não roda regras.
    if cli.emit_resolution {
        // Resolve o nome de superfície ao crate real via o registry do linter
        // (renomeação por-membro), para o emit refletir a resolução do próprio linter.
        let resolve_crate = |path: &Path, seg: &str| -> String {
            crate_registry
                .owner_of(path)
                .and_then(|o| o.renames.get(seg).cloned())
                .unwrap_or_else(|| seg.to_string())
        };
        print!(
            "{}",
            crystalline_lint::shell::cli::format_resolution(&all_parsed, &resolve_crate)
        );
        return;
    }

    // ── V7/V8/V11 post-reduce ─────────────────────────────────────────────────
    let v7_level = config.level_for("V7", ViolationLevel::Warning);
    let v11_level = config.level_for("V11", ViolationLevel::Error);

    if enabled.v7 {
        if let Some(ref ap) = all_prompts {
            all_violations.extend(orphan_prompt::check_orphans(&project_index, ap, v7_level));
        }
    }
    if enabled.v8 {
        all_violations.extend(alien_file::check_aliens(&project_index));
    }
    if enabled.v11 {
        all_violations.extend(dangling_contract::check_dangling_contracts(
            &project_index,
            v11_level,
        ));
    }
    if enabled.v22 {
        all_violations.extend(provenance_inventory::check_inventory(
            &all_parsed,
            &v21_config,
        ));
    }

    // ── Ordenação determinística ───────────────────────────────────────────────
    // Rayon não garante ordem — Fatal → Error → Warning, depois path, depois linha.
    crystalline_lint::shell::cli::sort_violations(&mut all_violations);

    // ── --fix-hashes branch ───────────────────────────────────────────────────
    if cli.fix_hashes {
        let rewriter = L3HashRewriter {
            nucleo_root: nucleo_root.clone(),
        };
        let entries = fix_hashes::plan(&all_violations, &rewriter);

        if cli.dry_run {
            if !cli.quiet {
                print!("{}", fix_hashes::format_plan(&entries));
            }
            return;
        }

        let unfixable = entries.iter().filter(|e| e.new_hash.is_none()).count();
        let results = fix_hashes::execute(&entries, &rewriter, false);

        // Re-run analysis to count remaining V5
        let remaining_v5 = {
            let reparser = MultiParser {
                rust: RustParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    crate_registry.clone(),
                ),
                ts: TsParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                py: PyParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                c: CParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                cpp: CppParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                zig: ZigParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                go: GoParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                java: JavaParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                elixir: ElixirParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
            };
            let rewalker = FileWalker::new(cli.path.clone(), config.clone());
            let (re_files, re_errors) = collect_walker_results(rewalker.files());
            let v5_only = EnabledChecks {
                v1: false,
                v2: false,
                v3: false,
                v4: false,
                v5: true,
                v6: false,
                v7: false,
                v8: false,
                v9: false,
                v10: false,
                v11: false,
                v12: false,
                v13: false,
                v14: false,
                v15: false,
                v16: false,
                v17: false,
                v18: false,
                v19: false,
                v20: false,
                v21: false,
                v22: false,
                v23: false,
                v24: false,
                v25: false,
            };
            let (violations, _, _) = run_pipeline(
                &re_files,
                &re_errors,
                &reparser,
                &v5_only,
                &l1_ports,
                &wiring_config,
                &l1_allowed,
                &config.analysis.lineage.strict_directories,
                config.check_test_imports.unwrap_or(false),
                &config.wildcard_exceptions,
                &v21_config,
                &semantic_levels,
                &UnknownCitationFreshness,
            );
            violations.iter().filter(|v| v.rule_id == "V5").count()
        };

        if !cli.quiet {
            print!(
                "{}",
                fix_hashes::format_results(&results, unfixable, remaining_v5)
            );
        }

        if remaining_v5 > 0 {
            process::exit(1);
        }
        return;
    }

    // ── --update-snapshot branch ──────────────────────────────────────────────
    if cli.update_snapshot {
        let rewriter = L3SnapshotWriter {
            nucleo_root: nucleo_root.clone(),
        };
        let entries = update_snapshot::plan(&all_violations, &all_parsed, &rewriter);

        if cli.dry_run {
            if !cli.quiet {
                print!("{}", update_snapshot::format_plan(&entries));
            }
            return;
        }

        let results = update_snapshot::execute(&entries, &rewriter, false);

        // Re-run to count remaining V6
        let remaining_v6 = {
            let reparser = MultiParser {
                rust: RustParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    crate_registry.clone(),
                ),
                ts: TsParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                py: PyParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                c: CParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                cpp: CppParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                zig: ZigParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                go: GoParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                java: JavaParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
                elixir: ElixirParser::new(
                    shared_prompt_reader.clone(),
                    shared_snapshot_reader.clone(),
                    config.clone(),
                    cli.path.clone(),
                ),
            };
            let rewalker = FileWalker::new(cli.path.clone(), config.clone());
            let (re_files, re_errors) = collect_walker_results(rewalker.files());
            let v6_only = EnabledChecks {
                v1: false,
                v2: false,
                v3: false,
                v4: false,
                v5: false,
                v6: true,
                v7: false,
                v8: false,
                v9: false,
                v10: false,
                v11: false,
                v12: false,
                v13: false,
                v14: false,
                v15: false,
                v16: false,
                v17: false,
                v18: false,
                v19: false,
                v20: false,
                v21: false,
                v22: false,
                v23: false,
                v24: false,
                v25: false,
            };
            let (violations, _, _) = run_pipeline(
                &re_files,
                &re_errors,
                &reparser,
                &v6_only,
                &l1_ports,
                &wiring_config,
                &l1_allowed,
                &config.analysis.lineage.strict_directories,
                config.check_test_imports.unwrap_or(false),
                &config.wildcard_exceptions,
                &v21_config,
                &semantic_levels,
                &UnknownCitationFreshness,
            );
            violations.iter().filter(|v| v.rule_id == "V6").count()
        };

        if !cli.quiet {
            print!(
                "{}",
                update_snapshot::format_results(&results, remaining_v6)
            );
        }

        if remaining_v6 > 0 {
            process::exit(1);
        }
        return;
    }

    // ── Normal output ─────────────────────────────────────────────────────────
    if !cli.quiet {
        let output = match cli.format {
            OutputFormat::Text => crystalline_lint::shell::cli::format_text(&all_violations),
            OutputFormat::Sarif => crystalline_lint::shell::cli::format_sarif(&all_violations),
            OutputFormat::N16Summary => {
                let stats = crystalline_lint::shell::n16_summary::collect_n16_stats(
                    &source_files,
                    &config.wildcard_exceptions,
                );
                crystalline_lint::shell::n16_summary::format_n16_summary(
                    &stats,
                    config.n16_min_sample_size(),
                )
            }
        };
        print!("{output}");
    }

    // ── Exit code ─────────────────────────────────────────────────────────────
    if crystalline_lint::shell::cli::should_fail(&all_violations, &cli.fail_on) {
        process::exit(1);
    }
}

// ── MultiParser — L4 composition root ────────────────────────────────────────

/// Selecciona parser por `file.language`. Linguagem não suportada →
/// `ParseError::UnsupportedLanguage`. Zero lógica de negócio — pura composição.
struct MultiParser {
    rust: RustParser<
        std::sync::Arc<crystalline_lint::infra::prompt_reader::CachedPromptReader<FsPromptReader>>,
        FsPromptSnapshotReader,
    >,
    ts: TsParser<
        std::sync::Arc<crystalline_lint::infra::prompt_reader::CachedPromptReader<FsPromptReader>>,
        FsPromptSnapshotReader,
    >,
    py: PyParser<
        std::sync::Arc<crystalline_lint::infra::prompt_reader::CachedPromptReader<FsPromptReader>>,
        FsPromptSnapshotReader,
    >,
    c: CParser<
        std::sync::Arc<crystalline_lint::infra::prompt_reader::CachedPromptReader<FsPromptReader>>,
        FsPromptSnapshotReader,
    >,
    cpp: CppParser<
        std::sync::Arc<crystalline_lint::infra::prompt_reader::CachedPromptReader<FsPromptReader>>,
        FsPromptSnapshotReader,
    >,
    zig: ZigParser<
        std::sync::Arc<crystalline_lint::infra::prompt_reader::CachedPromptReader<FsPromptReader>>,
        FsPromptSnapshotReader,
    >,
    go: GoParser<
        std::sync::Arc<crystalline_lint::infra::prompt_reader::CachedPromptReader<FsPromptReader>>,
        FsPromptSnapshotReader,
    >,
    java: JavaParser<
        std::sync::Arc<crystalline_lint::infra::prompt_reader::CachedPromptReader<FsPromptReader>>,
        FsPromptSnapshotReader,
    >,
    elixir: ElixirParser<
        std::sync::Arc<crystalline_lint::infra::prompt_reader::CachedPromptReader<FsPromptReader>>,
        FsPromptSnapshotReader,
    >,
}

impl LanguageParser for MultiParser {
    fn parse<'a>(&self, file: &'a SourceFile) -> Result<ParsedFile<'a>, ParseError> {
        match file.language {
            Language::Rust => self.rust.parse(file),
            Language::TypeScript => self.ts.parse(file),
            Language::Python => self.py.parse(file),
            Language::C => self.c.parse(file),
            Language::Cpp => self.cpp.parse(file),
            Language::Zig => self.zig.parse(file),
            Language::Go => self.go.parse(file),
            Language::Java => self.java.parse(file),
            Language::Elixir => self.elixir.parse(file),
            _ => Err(ParseError::UnsupportedLanguage {
                path: file.path.clone(),
                language: file.language.clone(),
            }),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Separates walker results into Ok (SourceFile) and Err (SourceError) buckets.
fn collect_walker_results(
    iter: impl Iterator<Item = Result<SourceFile, SourceError>>,
) -> (Vec<SourceFile>, Vec<SourceError>) {
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for result in iter {
        match result {
            Ok(sf) => files.push(sf),
            Err(e) => errors.push(e),
        }
    }
    (files, errors)
}

// ── L4 adapter: HashRewriter (L3 hash_writer → L2 port) ──────────────────────

struct L3HashRewriter {
    nucleo_root: PathBuf,
}

impl HashRewriter for L3HashRewriter {
    fn read_header(&self, source_path: &Path) -> Option<(String, String)> {
        hash_writer::read_header(source_path)
    }

    fn compute_hash(&self, prompt_path: &str) -> Option<String> {
        // Reuse hash calculation logic from FsPromptReader (with size limits)
        FsPromptReader {
            nucleo_root: self.nucleo_root.clone(),
        }
        .read_hash(prompt_path)
    }

    fn compute_source_hash(&self, source_path: &Path) -> Option<String> {
        hash_writer::compute_source_hash(source_path)
    }

    fn write_hash(&self, source_path: &Path, new_hash: &str) -> Result<(), String> {
        hash_writer::write_hash(source_path, new_hash)
    }

    fn write_prompt_meta(&self, prompt_path: &str, code_hash: &str) -> Result<(), String> {
        let full_path = self.nucleo_root.join(prompt_path);
        hash_writer::write_prompt_meta(&full_path, code_hash)
    }
}

// ── L4 adapter: SnapshotRewriter (L3 snapshot_writer → L2 port) ──────────────

struct L3SnapshotWriter {
    nucleo_root: PathBuf,
}

impl SnapshotRewriter for L3SnapshotWriter {
    fn serialize_snapshot(&self, interface: &PublicInterface<'_>) -> String {
        FsPromptSnapshotReader {
            nucleo_root: self.nucleo_root.clone(),
        }
        .serialize_snapshot(interface)
    }

    fn write_snapshot(&self, prompt_path: &str, snapshot: &str) -> Result<(), String> {
        let full_path = self.nucleo_root.join(prompt_path);
        snapshot_writer::write_snapshot(&full_path, snapshot)
    }
}

// ── Pipeline ──────────────────────────────────────────────────────────────────

/// Parses all source files and runs per-file checks via rayon Map-Reduce.
///
/// **Fase Map** (`par_iter`):
/// - `source_errors` → V0 Fatal + `LocalIndex::from_source_error()`
/// - `source_files`  → `parser.parse()` → `run_checks()` + `LocalIndex::from_parsed()`
/// - Cada thread produz `(Vec<Violation>, Option<ParsedFile>, LocalIndex)` independentemente
///
/// **Fase Reduce** (`fold` + `reduce`):
/// - `fold`: acumula por thread sem estado compartilhado
/// - `reduce`: funde os acumuladores de cada thread
/// - `ProjectIndex::merge` é associativa e comutativa — ordem de fusão não afeta resultado
///
/// `source_files` must outlive the returned vecs — `ParsedFile<'a>` borrows from
/// them (zero-copy, ADR-0004). `P` é `Sync` — sem campo mutable compartilhado.
fn run_pipeline<'a, P: LanguageParser + Sync>(
    source_files: &'a [SourceFile],
    source_errors: &'a [SourceError],
    parser: &P,
    enabled: &EnabledChecks,
    l1_ports: &L1Ports,
    wiring_config: &WiringConfig,
    l1_allowed: &L1AllowedExternalSet,
    strict_dirs: &[String],
    check_test_imports: bool,
    wildcard_exceptions: &HashMap<String, String>,
    v21_config: &crystalline_lint::rules::unsourced_constant::V21RuleConfig,
    semantic_levels: &[ViolationLevel; 3],
    citation_freshness: &(dyn CitationFreshnessResolver + Sync),
) -> (Vec<Violation<'a>>, Vec<ParsedFile<'a>>, ProjectIndex<'a>) {
    // Fase Map ─────────────────────────────────────────────────────────────────

    // V0: unreadable files — Fatal, never silenced
    let error_map = source_errors.par_iter().map(
        |err| -> (Vec<Violation<'a>>, Option<ParsedFile<'a>>, LocalIndex<'a>) {
            (
                vec![source_error_to_violation(err)],
                None,
                LocalIndex::from_source_error(),
            )
        },
    );

    // V1–V20 per file
    let file_map = source_files.par_iter().map(
        |source_file| -> (Vec<Violation<'a>>, Option<ParsedFile<'a>>, LocalIndex<'a>) {
            match parser.parse(source_file) {
                Ok(parsed) => {
                    let violations = run_checks(
                        &parsed,
                        enabled,
                        l1_ports,
                        wiring_config,
                        l1_allowed,
                        strict_dirs,
                        check_test_imports,
                        wildcard_exceptions,
                        v21_config,
                        semantic_levels,
                        citation_freshness,
                    );
                    let local = LocalIndex::from_parsed(&parsed);
                    (violations, Some(parsed), local)
                }
                Err(err) => (
                    vec![parse_error_to_violation(err)],
                    None,
                    LocalIndex::from_parse_error(),
                ),
            }
        },
    );

    // Fase Reduce ──────────────────────────────────────────────────────────────
    //
    // fold()   — acumula por thread; cada thread começa com acumulador vazio
    // reduce() — funde os acumuladores de threads distintas
    // ProjectIndex::merge() associativa e comutativa — ordem irrelevante

    error_map
        .chain(file_map)
        .fold(
            || {
                (
                    Vec::<Violation<'a>>::new(),
                    Vec::<ParsedFile<'a>>::new(),
                    ProjectIndex::<'a>::new(),
                )
            },
            |(mut viols, mut parsed_acc, mut idx), (v, pf, local)| {
                viols.extend(v);
                if let Some(p) = pf {
                    parsed_acc.push(p);
                }
                idx.merge_local(local);
                (viols, parsed_acc, idx)
            },
        )
        .reduce(
            || (Vec::new(), Vec::new(), ProjectIndex::new()),
            |(mut viols_a, mut parsed_a, idx_a), (viols_b, parsed_b, idx_b)| {
                viols_a.extend(viols_b);
                parsed_a.extend(parsed_b);
                (viols_a, parsed_a, idx_a.merge(idx_b))
            },
        )
}

// ── Rule dispatcher ───────────────────────────────────────────────────────────

fn run_checks<'a>(
    file: &ParsedFile<'a>,
    enabled: &EnabledChecks,
    l1_ports: &L1Ports,
    wiring_config: &WiringConfig,
    l1_allowed: &L1AllowedExternalSet,
    strict_dirs: &[String],
    check_test_imports: bool,
    wildcard_exceptions: &HashMap<String, String>,
    v21_config: &crystalline_lint::rules::unsourced_constant::V21RuleConfig,
    semantic_levels: &[ViolationLevel; 3],
    citation_freshness: &(dyn CitationFreshnessResolver + Sync),
) -> Vec<Violation<'a>> {
    let mut violations = Vec::new();
    let l1_allowed_lang = l1_allowed.for_language(&file.language);
    if enabled.v1 {
        violations.extend(prompt_header::check(file, strict_dirs));
    }
    if enabled.v2 {
        violations.extend(test_file::check(file));
    }
    if enabled.v3 {
        violations.extend(forbidden_import::check(file, check_test_imports));
    }
    if enabled.v4 {
        violations.extend(impure_core::check(file));
    }
    if enabled.v5 {
        violations.extend(prompt_drift::check(file));
    }
    if enabled.v6 {
        violations.extend(prompt_stale::check(file));
    }
    if enabled.v9 {
        violations.extend(pub_leak::check(file, l1_ports, check_test_imports));
    }
    if enabled.v10 {
        violations.extend(quarantine_leak::check(file));
    }
    if enabled.v12 {
        violations.extend(wiring_logic_leak::check(file, wiring_config));
    }
    if enabled.v13 {
        violations.extend(mutable_state_core::check(file));
    }
    if enabled.v14 {
        violations.extend(external_type_in_contract::check(
            file,
            l1_allowed_lang,
            check_test_imports,
        ));
    }
    if enabled.v15 {
        violations.extend(multi_prompt_header::check(file));
    }
    if enabled.v16 {
        violations.extend(wildcard_saturation::check(file, wildcard_exceptions));
    }
    if enabled.v17 {
        violations.extend(compound_guard::check(file));
    }
    if enabled.v18 {
        violations.extend(range_pattern::check(file));
    }
    if enabled.v19 {
        violations.extend(or_pattern_alternatives::check(file));
    }
    if enabled.v20 {
        violations.extend(deep_pattern_nesting::check(file));
    }
    if enabled.v21 {
        violations.extend(unsourced_constant::check(
            file,
            v21_config,
            citation_freshness,
        ));
    }
    if enabled.v23 {
        violations.extend(context_erasure::check(file, semantic_levels[0].clone()));
    }
    if enabled.v24 {
        violations.extend(semantic_field_loss::check(file, semantic_levels[1].clone()));
    }
    if enabled.v25 {
        violations.extend(decision_ownership::check(file, semantic_levels[2].clone()));
    }
    violations
}

// ── Error → Violation converters ─────────────────────────────────────────────

/// V0 — Unreadable source. Fatal, uses Cow::Owned (path not in any SourceFile).
fn source_error_to_violation(err: &SourceError) -> Violation<'static> {
    match err {
        SourceError::Unreadable { path, reason } => Violation {
            rule_id: "V0".to_string(),
            level: ViolationLevel::Fatal,
            message: format!("Arquivo ilegível: {reason}"),
            location: Location {
                path: Cow::Owned(path.clone()),
                line: 0,
                column: 0,
            },
        },
    }
}

/// Converts a parse error into a `Violation<'static>`.
fn parse_error_to_violation(err: ParseError) -> Violation<'static> {
    match err {
        ParseError::SyntaxError {
            path,
            line,
            column,
            message,
        } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Error,
            message: format!("Syntax error: {message}"),
            location: Location {
                path: Cow::Owned(path),
                line,
                column,
            },
        },
        ParseError::UnsupportedLanguage { path, language } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: format!("Unsupported language: {language:?}"),
            location: Location {
                path: Cow::Owned(path),
                line: 0,
                column: 0,
            },
        },
        ParseError::EmptySource { path } => Violation {
            rule_id: "PARSE".to_string(),
            level: ViolationLevel::Warning,
            message: "Empty source file skipped".to_string(),
            location: Location {
                path: Cow::Owned(path),
                line: 0,
                column: 0,
            },
        },
    }
}
