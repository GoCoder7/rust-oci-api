# Design

## Design Goals

- 기존 `EmailDelivery` struct의 public API를 깨지 않으면서 trait 추상화를 추가한다
- 의존 프로젝트에서 `dyn EmailSender`로 DI하여 mock/stub/dry-run 구현체를 주입할 수 있게 한다
- crate publish 워크플로를 skill로 정의하여 향후 릴리스를 체계화한다

## Non-Goals

- OCI Logging API 클라이언트 추가
- `ObjectStorage`, `Vault` 등 다른 서비스의 trait 추상화
- `async_trait` 의존 없이 구현 (nightly `async fn in trait`은 아직 edition 2024에서도 완전하지 않으므로 `async_trait` 사용)

## Architecture

### trait 정의

```rust
// src/services/email/trait.rs (신규)
use async_trait::async_trait;
use crate::error::Result;
use super::models::*;

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, email: Email) -> Result<SubmitEmailResponse>;
}
```

### 파일 구조 변경

```
src/services/email/
├── mod.rs          (기존, re-export 추가)
├── models.rs       (기존, 변경 없음)
├── client.rs       (기존, EmailSender impl 추가)
├── sender_trait.rs  (신규, EmailSender trait 정의)
└── api.rs          (기존)
```

### EmailDelivery에 trait impl

```rust
// client.rs
#[async_trait]
impl EmailSender for EmailDelivery {
    async fn send(&self, email: Email) -> Result<SubmitEmailResponse> {
        // 기존 send() 로직 이동
    }
}
```

### 의존 프로젝트에서의 사용 패턴

```rust
// techton-back 예시
pub struct OciEmail {
    sender: Arc<dyn EmailSender>,  // trait object
}

// production
OciEmail { sender: Arc::new(email_delivery) }

// test
OciEmail { sender: Arc::new(MockEmailSender::new()) }
```

## Data Flow

1. `EmailSender` trait이 `send()` 계약을 정의
2. `EmailDelivery`가 실제 OCI API 호출로 impl
3. 의존 프로젝트가 `Arc<dyn EmailSender>`로 주입
4. 테스트에서 mock 구현체 주입 → 호출 기록/검증

## Constraints

- `async_trait` crate 의존 추가 필요
- `EmailDelivery`의 기존 `pub async fn send()` 메서드는 trait impl로 이동하되, backward compat을 위해 inherent method도 유지하거나 re-export
- `get_email_configuration`, `list_senders`는 이번 trait 범위에서 제외 가능 (send만 추상화해도 테스트 목적 달성)

## Technical Decisions

1. **trait 범위**: `send()`만 포함 (최소 범위). `get_email_configuration`과 `list_senders`는 설정/조회용이라 mock 필요성 낮음
2. **`async_trait` 사용**: edition 2024에서도 `dyn` trait의 async fn은 `async_trait` 매크로가 필요
3. **breaking change 회피**: `EmailDelivery`에 inherent `send()` 메서드를 제거하지 않고, trait impl을 추가하는 방식. 기존 `email_delivery.send(email)` 호출이 그대로 동작
4. **version bump**: minor (0.5.0 → 0.6.0) — 새 trait 추가는 additive change

## Validation Strategy

1. 기존 단위 테스트 18개+ 통과 확인
2. 기존 integration test (`#[ignore]`) 통과 확인
3. `cargo clippy` 경고 없음
4. `cargo doc --no-deps` 빌드 성공
5. trait을 사용한 mock 예시 컴파일 확인 (doctest 또는 example)
