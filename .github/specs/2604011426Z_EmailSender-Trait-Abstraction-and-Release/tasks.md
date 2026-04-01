# Task Plan

## Task Principles

- Follow Red -> Green -> Refactor whenever possible.
- Do not start the next phase until the current phase is verified.
- Record actual execution results, not just intentions.

## Preparation

- [x] Baseline: `cargo test` 전체 통과 확인 (ignored 제외)
- [x] Baseline: `cargo clippy` 경고 수준 기록
- [x] Baseline: `cargo doc --no-deps` 빌드 성공 확인
- [x] `async_trait` crate 의존성 추가 여부 결정 (edition 2024 native async trait 지원 범위 확인)

## Step 1: `EmailSender` trait 정의

- [x] `Red`: trait을 사용하는 테스트 작성 (mock impl이 `send()`를 구현하고 호출 기록을 검증)
- [x] `Green`: `src/services/email/sender_trait.rs` 생성, `EmailSender` trait 정의
- [x] `Green`: `mod.rs`에서 re-export
- [x] `Refactor`: trait 시그니처와 Error 타입 정리

## Step 2: `EmailDelivery`에 trait impl

- [x] `Red`: `EmailDelivery`가 `EmailSender`를 impl하는지 컴파일 타임 검증 테스트
- [x] `Green`: `client.rs`에 `impl EmailSender for EmailDelivery` 추가
- [x] `Green`: 기존 inherent `send()` 메서드와의 호환성 확인
- [x] `Refactor`: 중복 코드 정리 (inherent method → trait method 위임 via `send_impl()`)

## Step 3: 기존 테스트 통과 확인

- [x] `cargo test` 전체 통과 (ignored 제외)
- [x] `cargo clippy` 경고 수준 baseline 이하

## Step 4: 문서 업데이트

- [x] `sender_trait.rs` doc comment 작성 (trait 설명, mock 예시)
- [x] README.md에 trait 사용법 섹션 추가
- [x] CHANGELOG.md 생성 (0.6.0 항목)
- [x] `cargo doc --no-deps` 빌드 성공

## Step 5: 릴리스 skill 정의

- [x] `.github/skills/release/SKILL.md` 작성
  - 버전 bump 절차 (Cargo.toml)
  - CHANGELOG 업데이트 규칙
  - `cargo publish --dry-run` 검증
  - `cargo publish` 실행
  - git tag 생성 및 push
- [x] skill 구조 검증 (`verify-skill`): PASS WITH WARNINGS (automatable steps without bin/ — non-blocking)

## Step 6: crate 릴리스

- [x] `Cargo.toml` version bump (0.5.0 → 0.6.0)
- [x] `cargo publish --dry-run` 성공
- [x] git commit & tag (`v0.6.0`)
- [x] `cargo publish` 실행
- [x] `crates.io`에서 새 버전 확인

## Completion Criteria

- [x] `EmailSender` trait이 정의되고 `EmailDelivery`에 impl되어 있다
- [x] 기존 API가 breaking change 없이 유지된다
- [x] 기존 테스트 전체 통과
- [x] README, CHANGELOG, doc comment가 업데이트되어 있다
- [x] `crates.io`에 0.6.0이 publish되어 있다
- [x] release skill이 `.github/skills/release/`에 존재한다
- [x] 후속 spec (`techton` OCI-Email-Test-Integration)에 handoff 기록

## Progress Log

### Preparation Results

- `cargo test`: 4 unit + 2 doctest 통과, 6 ignored integration tests
- `cargo clippy`: 2 pre-existing warnings (uninlined format args in object_storage) — baseline으로 기록
- `cargo doc --no-deps`: 빌드 성공
- `async_trait` 필요성 확인: Rust 1.88/edition 2024에서도 `dyn Trait` async dispatch에는 `async_trait` 필요

### Step 1-2 Results

- `sender_trait.rs` 생성 (67줄): `#[async_trait] pub trait EmailSender: Send + Sync { async fn send(...) }`
- doc comment에 실제 OCI 사용 예시 + mock 패턴 예시 포함
- `client.rs`에 `send_impl()` private 헬퍼 추출 → inherent `send()`와 trait `send()` 모두 위임
- `mod.rs`에 `pub mod sender_trait` + `pub use sender_trait::EmailSender` 추가
- `lib.rs`에 `pub use async_trait::async_trait` re-export 추가
- backward compatibility 확인: 기존 `email_delivery.send(email)` 호출 경로 변경 없음

### Step 3 Results

- `cargo test`: 4 unit + 4 doctest (신규 2개 포함) + 4 tempfile = 12 통과, 6 ignored
- `cargo clippy`: baseline과 동일 (2 pre-existing warnings, 신규 없음)

### Step 4 Results

- README.md: "Testing with `EmailSender` trait" 섹션 추가 (mock 예시 포함)
- CHANGELOG.md: 신규 생성, v0.6.0 Added/Changed 기록
- `cargo doc --no-deps`: 빌드 성공
- 설치 버전 `0.5.0` → `0.6.0`으로 갱신

### Step 5 Results

- `.github/skills/release/SKILL.md` 작성 (7단계 워크플로: version bump → changelog → dry-run → commit/tag → publish → push)
- `verify-skill`: PASS WITH WARNINGS — automatable steps without `bin/` 경고 1건 (non-blocking)

### Step 6 Results

- `Cargo.toml` version: `0.5.0` → `0.6.0`
- `cargo publish --dry-run`: 성공 (43.55s)
- git commit: `5c1c9ad` ("release: v0.6.0")
- git tag: `v0.6.0` (annotated)
- `cargo publish`: 성공 — `Published oci-api v0.6.0 at registry crates-io`
- `git push origin main` + `git push origin v0.6.0`: 완료

### Handoff

- 후속 spec: `techton/.github/specs/2604011426Z_OCI-Email-Test-Integration/`
- 전제: `oci-api = "0.6.0"` (crates.io에서 사용 가능)
- 핵심 전달: `Arc<dyn EmailSender>` DI 패턴으로 `OciEmail` 전환, `MockEmailSender` 테스트 헬퍼 구현
