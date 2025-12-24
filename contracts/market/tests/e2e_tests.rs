use ink_e2e::ContractsBackend;

type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

use market::{Estado, Marketplace, MarketplaceRef, Rol};

#[ink_e2e::test]
async fn e2e_flujo_compra_completo(mut client: Client) -> E2EResult<()> {
    // 1. Instanciar el contrato
    let mut constructor = MarketplaceRef::new();
    let contract = client
        .instantiate("market", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");

    let mut call_builder = contract.call_builder::<Marketplace>();

    // 2. Alice se registra como Vendedor
    let registrar_alice = call_builder.registrar(Rol::Vendedor);
    let result = client
        .call(&ink_e2e::alice(), &registrar_alice)
        .submit()
        .await
        .expect("registrar alice failed");
    assert!(result.return_value().is_ok());

    // 3. Bob se registra como Comprador
    let registrar_bob = call_builder.registrar(Rol::Comprador);
    let result = client
        .call(&ink_e2e::bob(), &registrar_bob)
        .submit()
        .await
        .expect("registrar bob failed");
    assert!(result.return_value().is_ok());

    // 4. Alice publica un producto
    let publicar = call_builder.publicar(
        String::from("Laptop"),
        String::from("Gaming Laptop"),
        1000,
        5,
        String::from("Electronics"),
    );
    let result = client
        .call(&ink_e2e::alice(), &publicar)
        .submit()
        .await
        .expect("publicar failed");
    let prod_id = result.return_value().expect("publicar logic error");

    // 5. Bob compra el producto
    let comprar = call_builder.comprar(prod_id, 1);
    let result = client
        .call(&ink_e2e::bob(), &comprar)
        .submit()
        .await
        .expect("comprar failed");
    let orden_id = result.return_value().expect("comprar logic error");

    // 6. Verificar estado de la orden (Pendiente)
    let get_orden = call_builder.obtener_orden(orden_id);
    let result = client
        .call(&ink_e2e::bob(), &get_orden)
        .submit()
        .await
        .expect("obtener_orden failed");
    let orden = result.return_value().unwrap();
    assert_eq!(orden.estado, Estado::Pendiente);

    // 7. Alice marca como Enviado
    let marcar_enviado = call_builder.marcar_enviado(orden_id);
    let result = client
        .call(&ink_e2e::alice(), &marcar_enviado)
        .submit()
        .await
        .expect("marcar_enviado failed");
    assert!(result.return_value().is_ok());

    // 8. Bob marca como Recibido
    let marcar_recibido = call_builder.marcar_recibido(orden_id);
    let result = client
        .call(&ink_e2e::bob(), &marcar_recibido)
        .submit()
        .await
        .expect("marcar_recibido failed");
    assert!(result.return_value().is_ok());

    // 9. Bob califica al vendedor
    let calificar = call_builder.calificar_vendedor(orden_id, 5);
    let result = client
        .call(&ink_e2e::bob(), &calificar)
        .submit()
        .await
        .expect("calificar failed");
    assert!(result.return_value().is_ok());

    // 10. Verificar reputación de Alice
    let alice_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Alice);
    let get_rep = call_builder.obtener_reputacion(alice_account);
    let result = client
        .call(&ink_e2e::bob(), &get_rep)
        .submit()
        .await
        .expect("obtener_reputacion failed");
    let rep = result.return_value().unwrap();
    assert_eq!(rep.como_vendedor, (5, 1)); // suma=5, count=1

    Ok(())
}

#[ink_e2e::test]
async fn e2e_flujo_cancelacion(mut client: Client) -> E2EResult<()> {
    // 1. Setup
    let mut constructor = MarketplaceRef::new();
    let contract = client
        .instantiate("market", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");

    let mut call_builder = contract.call_builder::<Marketplace>();

    // Registros
    let reg_alice = call_builder.registrar(Rol::Vendedor);
    client
        .call(&ink_e2e::alice(), &reg_alice)
        .submit()
        .await
        .expect("reg alice failed");

    let reg_bob = call_builder.registrar(Rol::Comprador);
    client
        .call(&ink_e2e::bob(), &reg_bob)
        .submit()
        .await
        .expect("reg bob failed");

    // Publicar
    let publicar = call_builder.publicar(
        String::from("TV"),
        String::from("4K"),
        200,
        10,
        String::from("Hogar"),
    );
    let result = client
        .call(&ink_e2e::alice(), &publicar)
        .submit()
        .await
        .expect("publicar failed");
    let pid = result.return_value().unwrap();

    // Comprar
    let comprar = call_builder.comprar(pid, 2);
    let result = client
        .call(&ink_e2e::bob(), &comprar)
        .submit()
        .await
        .expect("comprar failed");
    let oid = result.return_value().unwrap();

    // 2. Bob solicita cancelación
    let sol_cancel = call_builder.solicitar_cancelacion(oid);
    let result = client
        .call(&ink_e2e::bob(), &sol_cancel)
        .submit()
        .await
        .expect("sol_cancel failed");
    assert!(result.return_value().is_ok());

    // 3. Alice acepta cancelación
    let aceptar_cancel = call_builder.aceptar_cancelacion(oid);
    let result = client
        .call(&ink_e2e::alice(), &aceptar_cancel)
        .submit()
        .await
        .expect("aceptar_cancel failed");
    assert!(result.return_value().is_ok());

    // 4. Verificar stock restaurado
    let get_prod = call_builder.obtener_producto(pid);
    let result = client
        .call(&ink_e2e::alice(), &get_prod)
        .submit()
        .await
        .expect("obtener_producto failed");
    let prod = result.return_value().unwrap();
    assert_eq!(prod.stock, 10); // Stock restaurado

    Ok(())
}

#[ink_e2e::test]
async fn e2e_stock_insuficiente(mut client: Client) -> E2EResult<()> {
    let mut constructor = MarketplaceRef::new();
    let contract = client
        .instantiate("market", &ink_e2e::alice(), &mut constructor)
        .submit()
        .await
        .expect("instantiate failed");

    let mut call_builder = contract.call_builder::<Marketplace>();

    // Alice como Ambos
    let reg = call_builder.registrar(Rol::Ambos);
    client
        .call(&ink_e2e::alice(), &reg)
        .submit()
        .await
        .expect("reg failed");

    // Bob como Comprador
    let reg_bob = call_builder.registrar(Rol::Comprador);
    client
        .call(&ink_e2e::bob(), &reg_bob)
        .submit()
        .await
        .expect("reg bob failed");

    // Publicar con stock 1
    let publicar = call_builder.publicar(
        String::from("Item"),
        String::from("Desc"),
        10,
        1,
        String::from("Cat"),
    );
    let result = client
        .call(&ink_e2e::alice(), &publicar)
        .submit()
        .await
        .expect("publicar failed");
    let pid = result.return_value().unwrap();

    // Bob intenta comprar 2 (debe fallar)
    let comprar = call_builder.comprar(pid, 2);
    let result = client.call(&ink_e2e::bob(), &comprar).submit().await;

    // Debe fallar por stock insuficiente - la transacción completa falla
    assert!(
        result.is_err(),
        "Comprar con stock insuficiente debería fallar"
    );

    Ok(())
}
