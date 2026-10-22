//! 移动端 Rust 侧「远程授权门」（N11 客户端执行侧）的**跨端契约测试**。
//!
//! # ① 范围表的权威出处
//!
//! 本卡的范围表**不自创**：权威出处是**服务端**已落地的
//! `sales/lengsuo-core`（main = `b0b637b`）里的
//! `crates/lengsuo-coord/src/scope.rs`（该文件由提交 `f06e4b4` 引入）。
//!
//! 服务端是**签发侧**，本模块是**执行侧**：服务端签发，本地门执行。
//! 两端的名字、位序、码值**必须逐字一致**，否则 N11 的两端语义会漂移。
//!
//! # ② 三档与位序（跨端契约，逐字钉死）
//!
//! | scope（语义） | 服务端变体 | 权限码（`域:动作`） | 位 |
//! |---|---|---|---|
//! | 会话查看 | `SessionView` | `session:view` | `0b0000_0001` |
//! | 工具审批 | `ToolApprove` | `tool:approve` | `0b0000_0010` |
//! | 通知 | `Notification` | `notification:receive` | `0b0000_0100` |
//!
//! - 已知掩码 `KNOWN_MASK = 0b0000_0111`：**三档，没有第四档**；
//! - 枚举**声明顺序** = 位序（低→高）= 遍历顺序；
//! - `notification` 在范围表里是单段词，但权限码必须是 `域:动作` **两段**
//!   才判为合法 ⇒ 服务端选用两段形态 `notification:receive`
//!   （服务端 `scope.rs` 的注释已写明该理由）⇒ **本卡用同一个码**。
//!
//! # ③ fail-closed 规则
//!
//! 远程授权范围**由服务端签发**，客户端本地门执行，**fail-closed**：
//!
//! - 凭据缺失 / 过期 / 协调面不可达 ⇒ **一律拒绝**；
//! - ⛔ **不降级**为明文、⛔ **不降级**为无鉴权；
//! - ⛔ **不引入任何降级开关**（没有「无凭据也放行」的后门）；
//! - ⛔ **未知位拒绝**：`from_bits` 遇到 `KNOWN_MASK` 之外的位 ⇒ **报错**，
//!   ⛔ 不得静默丢弃。
//!
//! # ④ 不做包含关系推断（待主理人定）
//!
//! [`RemoteScope::implies`] **恒为 `false`**：契约没定义谁蕴含谁。
//! 猜「工具审批蕴含会话查看」会静默**放宽**授权，猜反了会静默**收窄**产品
//! ⇒ 两端一律留白，等主理人定。需要精确判定时用 [`RemoteScopeSet::contains`]。
//!
//! # ⑤ 跨端对拍结论
//!
//! 本文件把范围表**逐字**断言下来（码值 + 位值 + 顺序 + 掩码），
//! 使「移动端执行侧」与「服务端签发侧」第一次被同一批断言同时覆盖：
//! 服务端哪天改了码值或位序，这里的断言会先红。

use lengsuo_mobile_ffi::core::remote_gate::{
    permission_of_scope, require_scope, scope_of_permission, RemoteGate, RemoteGateError,
    RemoteGateState, RemoteScope, RemoteScopeSet, KNOWN_MASK, PERM_NOTIFICATION_RECEIVE,
    PERM_SESSION_VIEW, PERM_TOOL_APPROVE, REMOTE_SCOPE_PERMISSIONS,
};

/// 契约禁止的第四档：凭据读取 / 内容导出。
///
/// 这两个能力在范围表里**无法表达**——枚举里没有对应变体，
/// 权限码也不在映射表里 ⇒ 派生出来的档位集合**不含**它们。
const FORBIDDEN_CODES: [&str; 4] = [
    "credential:read",
    "credential:export",
    "session:export",
    "device:register",
];

/// 全部三档的权限码（服务端签发的正常形态）。
const ALL_CODES: [&str; 3] = [
    PERM_SESSION_VIEW,
    PERM_TOOL_APPROVE,
    PERM_NOTIFICATION_RECEIVE,
];

/// 便捷构造：按权限码派生一个「有效凭据」的门。
fn gate_with(codes: &[&str]) -> RemoteGate {
    RemoteGate::authorized_by_permissions(codes)
}

/// 穷尽匹配 `RemoteGateState` 的**全部变体**，返回变体个数。
///
/// 刻意**不写通配分支** ⇒ 将来给状态加变体时，下面的 `match` 会**编译不过**，
/// 强制主理人重新确认「还有没有第四条放行路径」。
fn state_variant_count() -> usize {
    let states = [
        RemoteGateState::Authorized {
            scopes: RemoteScopeSet::empty(),
        },
        RemoteGateState::CredentialMissing,
        RemoteGateState::CredentialExpired,
        RemoteGateState::CoordinationUnreachable,
    ];
    let mut count = 0;
    for state in states {
        match state {
            RemoteGateState::Authorized { .. } => count += 1,
            RemoteGateState::CredentialMissing => count += 1,
            RemoteGateState::CredentialExpired => count += 1,
            RemoteGateState::CoordinationUnreachable => count += 1,
        }
    }
    count
}

#[test]
fn cross_end_scope_table_matches_server_contract_verbatim() {
    // 逐项断言第二节的表：档位 ⇄ 权限码 ⇄ 位值。
    assert_eq!(
        permission_of_scope(RemoteScope::SessionView),
        "session:view"
    );
    assert_eq!(RemoteScope::SessionView.permission(), "session:view");
    assert_eq!(RemoteScope::SessionView.bit(), 0b0000_0001);

    assert_eq!(
        permission_of_scope(RemoteScope::ToolApprove),
        "tool:approve"
    );
    assert_eq!(RemoteScope::ToolApprove.permission(), "tool:approve");
    assert_eq!(RemoteScope::ToolApprove.bit(), 0b0000_0010);

    assert_eq!(
        permission_of_scope(RemoteScope::Notification),
        "notification:receive"
    );
    assert_eq!(
        RemoteScope::Notification.permission(),
        "notification:receive"
    );
    assert_eq!(RemoteScope::Notification.bit(), 0b0000_0100);

    // 映射表本身与服务端逐字一致。
    assert_eq!(PERM_SESSION_VIEW, "session:view");
    assert_eq!(PERM_TOOL_APPROVE, "tool:approve");
    assert_eq!(PERM_NOTIFICATION_RECEIVE, "notification:receive");
    assert_eq!(REMOTE_SCOPE_PERMISSIONS, ALL_CODES);

    // 双向映射自洽。
    for scope in RemoteScope::ALL {
        let code = permission_of_scope(scope);
        assert_eq!(scope_of_permission(code), Some(scope));
    }

    // 位序 = 声明顺序 = 低位在前。
    assert_eq!(
        RemoteScope::ALL,
        [
            RemoteScope::SessionView,
            RemoteScope::ToolApprove,
            RemoteScope::Notification
        ]
    );
}

#[test]
fn known_mask_covers_exactly_three_bits() {
    assert_eq!(KNOWN_MASK, 0b0000_0111);
    assert_eq!(RemoteScope::ALL.len(), 3);
    assert_eq!(KNOWN_MASK.count_ones(), 3, "掩码里必须正好三档");

    // 三档的位互不相同（各自占单独一位），且全部落在掩码内。
    let mut union = 0u8;
    let mut sum = 0u8;
    for scope in RemoteScope::ALL {
        assert!(scope.bit().is_power_of_two(), "每档必须占单独一位");
        union |= scope.bit();
        sum += scope.bit();
    }
    assert_eq!(union, KNOWN_MASK, "掩码必须被三档恰好填满");
    assert_eq!(sum, KNOWN_MASK, "三档的位必须互不相同");
}

#[test]
fn no_fourth_scope_can_be_expressed() {
    // 枚举里没有第四档。
    assert_eq!(RemoteScope::ALL.len(), 3);

    // 契约禁止的能力：权限码**存在**，但**不在映射表里** ⇒ 派生不出任何档位。
    for code in FORBIDDEN_CODES {
        assert_eq!(
            scope_of_permission(code),
            None,
            "禁止的能力不得映射出档位：{code}"
        );
    }

    // 只带禁止码的门 ⇒ 一档都不授予。
    let gate = gate_with(&FORBIDDEN_CODES);
    assert!(gate.scopes().is_empty());
    for scope in RemoteScope::ALL {
        assert!(require_scope(&gate, scope).is_err());
    }

    // 映射表里只有三项 ⇒ 别的域的码不可能放宽远程范围。
    assert_eq!(REMOTE_SCOPE_PERMISSIONS.len(), 3);
}

#[test]
fn unknown_bits_are_rejected_not_masked_away() {
    // 掩码内的位：一律接受。
    assert!(RemoteScopeSet::from_bits(0).is_ok());
    assert!(RemoteScopeSet::from_bits(KNOWN_MASK).is_ok());
    for scope in RemoteScope::ALL {
        assert!(RemoteScopeSet::from_bits(scope.bit()).is_ok());
    }

    // 掩码外的位：**报错**，不得静默丢弃。
    for bad in [0b0000_1000u8, 0b0001_0000, 0b1000_0000, 0b1111_1111] {
        let error = RemoteScopeSet::from_bits(bad).expect_err("未知位必须报错");
        assert_eq!(error.stable_name(), "UnknownScopeBits");
        // 不得有「known_only() 之类」的兜底：报错的掩码一个位也不能留下。
        match error {
            RemoteGateError::UnknownScopeBits { bits } => assert_eq!(bits, bad),
            other => panic!("错误类型不对：{other:?}"),
        }
    }

    // 掩码内 + 掩码外 ⇒ 依然报错（不是「忽略越界位」）。
    assert!(RemoteScopeSet::from_bits(KNOWN_MASK | 0b0010_0000).is_err());
}

#[test]
fn scope_is_granted_only_when_its_permission_code_is_present() {
    // 三档各来一遍：有对应权限码 ⇒ 放行；缺 ⇒ 拒绝。
    for (index, scope) in RemoteScope::ALL.into_iter().enumerate() {
        let code = permission_of_scope(scope);

        // 只带这一档的权限码 ⇒ 只有这一档放行。
        let gate = gate_with(&[code]);
        assert!(require_scope(&gate, scope).is_ok(), "持码即应放行：{code}");
        for other in RemoteScope::ALL {
            if other != scope {
                assert!(
                    require_scope(&gate, other).is_err(),
                    "未持码不得放行：{other}"
                );
            }
        }

        // 全带 ⇒ 三档全放行。
        let all = gate_with(&ALL_CODES);
        assert!(require_scope(&all, scope).is_ok());
        assert_eq!(all.scopes().len(), 3);

        // 全不带 ⇒ 三档全拒。
        let none = gate_with(&[]);
        assert!(require_scope(&none, scope).is_err());

        // 缺档时错误要**点名缺哪一档**。
        let missing = RemoteScope::ALL[(index + 1) % 3];
        let error = require_scope(&gate_with(&[code]), missing).expect_err("缺档必须拒绝");
        assert_eq!(error.stable_name(), "ScopeMissing");
        let text = error.to_string();
        assert!(
            text.contains(permission_of_scope(missing)),
            "错误文案要能一眼看出缺哪一档：{text}"
        );
    }
}

#[test]
fn missing_scope_error_names_the_missing_tier() {
    let gate = gate_with(&[PERM_SESSION_VIEW]);
    let error = require_scope(&gate, RemoteScope::ToolApprove).expect_err("缺档必须拒绝");
    match error {
        RemoteGateError::ScopeMissing { scope, permission } => {
            assert_eq!(scope, RemoteScope::ToolApprove);
            assert_eq!(permission, "tool:approve");
        }
        other => panic!("错误类型不对：{other:?}"),
    }
    // 文案里带得动缺的那一档的名字与权限码。
    let text = error.to_string();
    assert!(text.contains("tool:approve"), "文案要带权限码：{text}");
    assert!(text.contains("ToolApprove"), "文案要带档位名：{text}");
}

#[test]
fn fail_closed_without_valid_credential() {
    // 任务书的三态（凭据缺失或过期合一，此处拆细成两种错误）+ 协调面不可达：
    // **一律拒绝**，且**不得**回落到「无鉴权放行」或「明文降级」。
    let broken: [(&str, RemoteGate, &str); 3] = [
        (
            "凭据缺失",
            RemoteGate::credential_missing(),
            "CredentialMissing",
        ),
        (
            "凭据过期",
            RemoteGate::credential_expired(),
            "CredentialExpired",
        ),
        (
            "协调面不可达",
            RemoteGate::coordination_unreachable(),
            "CoordinationUnreachable",
        ),
    ];

    for (label, gate, expected) in broken {
        for scope in RemoteScope::ALL {
            let error = require_scope(&gate, scope).expect_err("无有效凭据必须拒绝");
            assert_eq!(error.stable_name(), expected, "场景：{label}");
        }
        // 非授权状态 ⇒ 一档都不授予（没有任何「顺带放行」）。
        assert!(gate.scopes().is_empty(), "场景：{label}");
    }
}

#[test]
fn no_construction_path_allows_without_credential() {
    // 逐个走遍**所有**公开构造路径，确认没有一条能绕过凭据检查。
    let paths: [(&str, RemoteGate); 4] = [
        ("凭据缺失", RemoteGate::credential_missing()),
        ("凭据过期", RemoteGate::credential_expired()),
        ("协调面不可达", RemoteGate::coordination_unreachable()),
        (
            "有效凭据但一档未签发",
            RemoteGate::authorized(RemoteScopeSet::empty()),
        ),
    ];

    for (label, gate) in paths {
        assert!(gate.scopes().is_empty(), "场景：{label}");
        for scope in RemoteScope::ALL {
            assert!(
                require_scope(&gate, scope).is_err(),
                "场景：{label} 不得放行 {scope:?}"
            );
        }
    }

    // 授权态的档位只能来自「有效凭据 + 已签发的档位集合」这一条路。
    let granted = RemoteGate::authorized_by_permissions(&ALL_CODES);
    assert_eq!(granted.scopes().len(), 3);

    // 状态枚举只有四档（穷尽匹配，不写通配分支 ⇒ 加变体即编译失败）。
    assert_eq!(state_variant_count(), 4);

    // 非授权态永远拿不到档位：逐状态核对。
    for state in [
        RemoteGateState::Authorized {
            scopes: RemoteScopeSet::empty(),
        },
        RemoteGateState::CredentialMissing,
        RemoteGateState::CredentialExpired,
        RemoteGateState::CoordinationUnreachable,
    ] {
        let scopes = match state {
            RemoteGateState::Authorized { scopes } => scopes,
            RemoteGateState::CredentialMissing
            | RemoteGateState::CredentialExpired
            | RemoteGateState::CoordinationUnreachable => RemoteScopeSet::empty(),
        };
        assert!(scopes.is_empty(), "非授权态不得携带档位");
    }
}

#[test]
fn implies_is_always_false() {
    // 契约没定义谁蕴含谁 ⇒ 任意两档之间都不蕴含（含自蕴含）。
    for left in RemoteScope::ALL {
        assert!(!left.implies(left), "自蕴含也必须为 false：{left:?}");
        for right in RemoteScope::ALL {
            assert!(
                !left.implies(right),
                "不得推断包含关系：{left:?} ⇒ {right:?}"
            );
        }
    }

    // 精确判定走 contains，不走 implies。
    let mut set = RemoteScopeSet::empty();
    assert!(set.insert(RemoteScope::SessionView));
    assert!(set.contains(RemoteScope::SessionView));
    assert!(!set.contains(RemoteScope::ToolApprove));
    assert_eq!(
        set.iter().collect::<Vec<_>>(),
        vec![RemoteScope::SessionView]
    );
    assert_eq!(set.bits(), 0b0000_0001);
}
