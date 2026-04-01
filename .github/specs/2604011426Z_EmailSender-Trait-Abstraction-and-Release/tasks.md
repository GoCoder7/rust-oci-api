# Task Plan

## Task Principles

- Follow Red -> Green -> Refactor whenever possible.
- Do not start the next phase until the current phase is verified.
- Record actual execution results, not just intentions.

## Preparation

- [ ] Baseline: `cargo test` 전체 통과 확인 (ignored 제외)
- [ ] Baseline: `cargo clippy` 경고 수준 기록
- [ ] Baseline: `cargo doc --no-deps` 빌드 성공 확인
- [ ] `async_trait` crate 의존성 추가 여부 결정 (edition 2024 native async trait 지원 범위 확인)

## Step 1: `EmailSender` trait 정의

- [ ] `Red`: trait을 사용하는 테스트 작성 (mock impl이 `send()`를 구현하고 호출 기록을 검증)
- [ ] `Green`: `src/services/email/sender_trait.rs` 생성, `EmailSender` trait 정의
- [ ] `Green`: `mod.rs`에서 re-export
- [ ] `Refactor`: trait 시그니처와 Error 타입 정리

## Step 2: `EmailDelivery`에 trait impl

- [ ] `Red`: `EmailDelivery`가 `EmailSender`를 impl하는지 컴파일 타임 검증 테스트
- [ ] `Green`: `client.rs`에 `impl EmailSender for EmailDelivery` 추가
- [ ] `Green`: 기존 inherent `send()` 메서드와의 호환성 확인
- [ ] `Refactor`: 중복 코드 정리 (inherent method → trait method 위임)

## Step 3: 기존 테스트 통과 확인

- [ ] `cargo test` 전체 통과 (ignored 제외)
- [ ] `cargo test --test real_oci_integration_test -- --ignored` 통과 (OCI credential 있는 환경에서)
- [ ] `cargo clippy` 경고 수준 baseline 이하

## Step 4: 문서 업데이트

- [ ] `sender_trait.rs` doc comment 작성 (trait 설명, mock 예시)
- [ ] README.md에 trait 사용법 섹션 추가
- [ ] CHANGELOG.md 업데이트 (0.6.0 항목)
- [ ] `cargo doc --no-deps` 빌드 성공

## Step 5: 릴리스 skill 정의

- [ ] `.github/skills/release/SKILL.md` 작성
  - 버전 bump 절차 (Cargo.toml)
  - CHANGELOG 업데이트 규칙
  - `cargo publish --dry-run` 검증
  - `cargo publish` 실행
  - git tag 생성 및 push
- [ ] skill 구조 검증 (`verify-skill`)

## Step 6: crate 릴리스

- [ ] `Cargo.toml` version bump (0.5.0 → 0.6.0)
- [ ] `cargo publish --dry-run` 성공
- [ ] git commit & tag (`v0.6.0`)
- [ ] `cargo publish` 실행
- [ ] `crates.io`에서 새 버전 확인

## Completion Criteria

- [ ] `EmailSender` trait이 정의되고 `EmailDelivery`에 impl되어 있다
- [ ] 기존 API가 breaking change 없이 유지된다
- [ ] 기존 테스트 전체 통과
- [ ] README, CHANGELOG, doc comment가 업데이트되어 있다
- [ ] `crates.io`에 0.6.0이 publish되어 있다
- [ ] release skill이 `.github/skills/release/`에 존재한다
- [ ] 후속 spec (`techton` OCI-Email-Test-Integration)에 handoff 기록

## Progress Log

### Preparation Results

-

### Step 1 Results

-
