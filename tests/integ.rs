use anyhow::bail;
use core::f32;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use crate::common::{TestContext, default_rules};

mod common;

#[tokio::test(flavor = "multi_thread")]
async fn network_01_tcp_connect() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    tokio::net::TcpSocket::new_v4()?
        .connect(ctx.server_addr)
        .await?;
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn network_02_tcp_connect_and_close() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    tokio::net::TcpSocket::new_v4()?
        .connect(ctx.server_addr)
        .await?
        .shutdown()
        .await?;
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn network_03_ws_upgrade() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    tokio_tungstenite::connect_async(format!("ws://{}", ctx.server_addr))
        .await?
        .0
        .close(None)
        .await?;
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn auth_04_one_client_can_authenticate() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client = dronoid::Client::connect(ctx.server_addr).await?;
    let response = client.authenticate("Player").await?;
    assert!(response.result);
    assert_eq!("Welcome", response.text);
    assert!(0. != response.spawn_point.0);
    assert!(0. != response.spawn_point.1);
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn auth_05_two_clients_can_authenticate() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client1 = dronoid::Client::connect(ctx.server_addr).await?;
    let mut client2 = dronoid::Client::connect(ctx.server_addr).await?;
    let response1 = client1.authenticate("Player1").await?;
    let response2 = client2.authenticate("Player2").await?;
    assert!(response2.result);
    assert_eq!("Welcome", response2.text);
    assert!(0. != response2.spawn_point.0);
    assert!(0. != response2.spawn_point.1);
    assert!(response1.spawn_point.0 != response2.spawn_point.0);
    assert!(response1.spawn_point.1 != response2.spawn_point.1);
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn auth_06_two_clients_same_name_fails() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client1 = dronoid::Client::connect(ctx.server_addr).await?;
    let mut client2 = dronoid::Client::connect(ctx.server_addr).await?;
    let _ = client1.authenticate("Player").await?;
    let response = client2.authenticate("Player").await?;
    assert!(!response.result);
    assert_eq!("Already playing / name taken", response.text);
    assert_eq!((0., 0.), response.spawn_point);
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn gameplay_07_one_player_place_factory_in_zone() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client = dronoid::Client::authenticated(ctx.server_addr, "Player").await?;
    client
        .send_action(dronoid::protocol::Action::PlaceFactory(client.spawn_point))
        .await?;
    let response = client.until_response().await?;
    if let dronoid::protocol::Response::PlaceFactory { result } = response {
        assert!(result);
    } else {
        bail!("Not a place factory response");
    }
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn gameplay_08_one_player_place_factory_one_response_only() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client = dronoid::Client::authenticated(ctx.server_addr, "Player").await?;
    client
        .send_action(dronoid::protocol::Action::PlaceFactory(client.spawn_point))
        .await?;
    client.until_response().await?;
    assert!(
        tokio::time::timeout(Duration::from_secs_f64(0.5), client.until_response())
            .await
            .is_err()
    );
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn gameplay_09_one_player_place_factory_out_of_zone_fails() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client = dronoid::Client::authenticated(ctx.server_addr, "Player").await?;
    let mut place_position = client.spawn_point;
    place_position.0 += 1000.;
    client
        .send_action(dronoid::protocol::Action::PlaceFactory(place_position))
        .await?;
    let response = client.until_response().await?;
    if let dronoid::protocol::Response::PlaceFactory { result } = response {
        assert!(!result);
    } else {
        bail!("Not a place factory response");
    }
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn gameplay_10_two_players_cant_place_factory_on_other_zone() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client1 = dronoid::Client::authenticated(ctx.server_addr, "Player1").await?;
    let mut client2 = dronoid::Client::authenticated(ctx.server_addr, "Player2").await?;
    client1
        .send_action(dronoid::protocol::Action::PlaceFactory(client2.spawn_point))
        .await?;
    let response1 = client1.until_response().await?;
    client2
        .send_action(dronoid::protocol::Action::PlaceFactory(client1.spawn_point))
        .await?;
    let response2 = client2.until_response().await?;

    if let dronoid::protocol::Response::PlaceFactory { result } = response1 {
        assert!(!result);
    } else {
        bail!("Not a place factory response");
    }
    if let dronoid::protocol::Response::PlaceFactory { result } = response2 {
        assert!(!result);
    } else {
        bail!("Not a place factory response");
    }
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn gameplay_11_one_player_receive_first_state() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client = dronoid::Client::authenticated(ctx.server_addr, "Player").await?;
    let first_state = client.until_state().await?;
    assert!(first_state.entities_in_zone.len() > 0);
    assert_eq!(
        1,
        first_state.entities_in_zone.iter().fold(0, |a, x| {
            if x.kind == dronoid::protocol::Kind::Spawn {
                a + 1
            } else {
                a
            }
        })
    );
    let entity = first_state
        .entities_in_zone
        .iter()
        .find(|x| x.kind == dronoid::protocol::Kind::Spawn)
        .unwrap();
    assert_eq!(entity.pos.0, client.spawn_point.0);
    assert_eq!(entity.pos.1, client.spawn_point.1);
    assert_eq!(entity.kind, dronoid::protocol::Kind::Spawn);
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn gameplay_12_one_player_receive_two_states() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client = dronoid::Client::authenticated(ctx.server_addr, "Player").await?;
    client.until_state().await?;
    let second_state = client.until_state().await?;
    assert!(second_state.entities_in_zone.len() > 0);
    assert_eq!(
        1,
        second_state.entities_in_zone.iter().fold(0, |a, x| {
            if x.kind == dronoid::protocol::Kind::Spawn {
                a + 1
            } else {
                a
            }
        })
    );
    let entity = second_state
        .entities_in_zone
        .iter()
        .find(|x| x.kind == dronoid::protocol::Kind::Spawn)
        .unwrap();
    assert_eq!(entity.pos.0, client.spawn_point.0);
    assert_eq!(entity.pos.1, client.spawn_point.1);
    assert!(
        10 < second_state.entities_in_zone.iter().fold(0, |a, x| {
            if x.kind == dronoid::protocol::Kind::Mineral {
                a + 1
            } else {
                a
            }
        })
    );
    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn gameplay_13_two_players_receive_first_state() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client1 = dronoid::Client::authenticated(ctx.server_addr, "Player1").await?;
    let mut client2 = dronoid::Client::authenticated(ctx.server_addr, "Player2").await?;
    let first_state1 = client1.until_state().await?;
    let first_state2 = client2.until_state().await?;

    assert!(first_state1.entities_in_zone.len() > 0);
    let entity1 = first_state1
        .entities_in_zone
        .iter()
        .find(|x| x.kind == dronoid::protocol::Kind::Spawn)
        .unwrap();
    assert_eq!(entity1.pos.0, client1.spawn_point.0);
    assert_eq!(entity1.pos.1, client1.spawn_point.1);

    assert!(first_state2.entities_in_zone.len() > 0);
    let entity2 = first_state2
        .entities_in_zone
        .iter()
        .find(|x| x.kind == dronoid::protocol::Kind::Spawn)
        .unwrap();
    assert_eq!(entity2.pos.0, client2.spawn_point.0);
    assert_eq!(entity2.pos.1, client2.spawn_point.1);

    ctx.teardown().await
}

#[tokio::test(flavor = "multi_thread")]
async fn gameplay_14_place_factory_circle() -> anyhow::Result<()> {
    let rules = default_rules();
    let ctx = TestContext::setup(rules.clone()).await?;
    let mut client = dronoid::Client::authenticated(ctx.server_addr, "Player").await?;
    let mut angle = -f32::consts::PI;
    while angle < f32::consts::PI {
        let response = tokio::time::timeout(
            Duration::from_secs_f32(1.),
            client.action(dronoid::protocol::Action::PlaceFactory((
                client.spawn_point.0 + rules.zone_extensions.factory * (angle.cos()),
                client.spawn_point.1 + rules.zone_extensions.factory * (angle.sin()),
            ))),
        )
        .await??;
        if let dronoid::protocol::Response::PlaceFactory { result } = response {
            assert!(result);
        } else {
            bail!("Not the expected PlaceFactory response");
        }
        angle += f32::consts::TAU / 10.;
    }
    while angle < f32::consts::PI {
        let response = tokio::time::timeout(
            Duration::from_secs_f32(1.),
            client.action(dronoid::protocol::Action::PlaceFactory((
                client.spawn_point.0
                    + (rules.zone_extensions.factory + f32::EPSILON) * (angle).cos(),
                client.spawn_point.1
                    + (rules.zone_extensions.factory + f32::EPSILON) * (angle).sin(),
            ))),
        )
        .await??;
        if let dronoid::protocol::Response::PlaceFactory { result } = response {
            assert!(!result);
        } else {
            bail!("Not the expected PlaceFactory response");
        }
        angle += f32::consts::TAU / 10.;
    }
    ctx.teardown().await
}
