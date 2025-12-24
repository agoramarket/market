use ink_e2e::ContractsBackend;

type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

use market::{Marketplace, MarketplaceRef, Rol};
use reports::{Reportes, ReportesRef, UsuarioConReputacion, ProductoVendido, EstadisticasCategoria, Error as ReportError};

#[ink_e2e::test]
async fn e2e_generacion_reportes(mut client: Client) -> E2EResult<()> {
    // 1. Deploy Market
    let mut market_constructor = MarketplaceRef::new();
    let market_contract = client
        .instantiate("market", &ink_e2e::alice(), &mut market_constructor)
        .submit()
        .await
        .expect("instantiate market failed");
    
    let market_acc_id = market_contract.account_id;
    let mut market_call = market_contract.call_builder::<Marketplace>();

    // 2. Deploy Reports (vinculado al Market)
    let mut reports_constructor = ReportesRef::new(market_acc_id);
    let reports_contract = client
        .instantiate("reports", &ink_e2e::alice(), &mut reports_constructor)
        .submit()
        .await
        .expect("instantiate reports failed");
    
    let mut reports_call = reports_contract.call_builder::<Reportes>();

    // 3. Generar datos en Market
    // Alice vende
    let reg_alice = market_call.registrar(Rol::Vendedor);
    client.call(&ink_e2e::alice(), &reg_alice).submit().await.expect("reg alice failed");

    // Bob compra
    let reg_bob = market_call.registrar(Rol::Comprador);
    client.call(&ink_e2e::bob(), &reg_bob).submit().await.expect("reg bob failed");

    // Charlie compra
    let reg_charlie = market_call.registrar(Rol::Comprador);
    client.call(&ink_e2e::charlie(), &reg_charlie).submit().await.expect("reg charlie failed");

    // Publicar producto
    let publicar = market_call.publicar(
        String::from("Laptop"),
        String::from("Gamer"),
        100,
        10,
        String::from("Tech"),
    );
    let result = client.call(&ink_e2e::alice(), &publicar).submit().await.expect("publicar failed");
    let prod_id = result.return_value().unwrap();

    // Bob compra 2
    let comprar_bob = market_call.comprar(prod_id, 2);
    let result = client.call(&ink_e2e::bob(), &comprar_bob).submit().await.expect("comprar bob failed");
    let oid_bob = result.return_value().unwrap();

    // Completar orden Bob
    let env_bob = market_call.marcar_enviado(oid_bob);
    client.call(&ink_e2e::alice(), &env_bob).submit().await.expect("env bob failed");

    let rec_bob = market_call.marcar_recibido(oid_bob);
    client.call(&ink_e2e::bob(), &rec_bob).submit().await.expect("rec bob failed");

    let calif_bob = market_call.calificar_vendedor(oid_bob, 5);
    client.call(&ink_e2e::bob(), &calif_bob).submit().await.expect("calif bob failed");

    // Charlie compra 3
    let comprar_charlie = market_call.comprar(prod_id, 3);
    let result = client.call(&ink_e2e::charlie(), &comprar_charlie).submit().await.expect("comprar charlie failed");
    let oid_charlie = result.return_value().unwrap();

    // Completar orden Charlie
    let env_charlie = market_call.marcar_enviado(oid_charlie);
    client.call(&ink_e2e::alice(), &env_charlie).submit().await.expect("env charlie failed");

    let rec_charlie = market_call.marcar_recibido(oid_charlie);
    client.call(&ink_e2e::charlie(), &rec_charlie).submit().await.expect("rec charlie failed");

    let calif_charlie = market_call.calificar_vendedor(oid_charlie, 3);
    client.call(&ink_e2e::charlie(), &calif_charlie).submit().await.expect("calif charlie failed");

    // 4. Consultar Reportes

    // Resumen General
    let resumen_msg = reports_call.resumen_general();
    let result = client.call(&ink_e2e::alice(), &resumen_msg).submit().await.expect("resumen failed");
    let resumen = result.return_value();
    
    // Verificar datos del resumen (tupla: usuarios, productos, ordenes, completadas)
    assert!(resumen.0 >= 3); // total_usuarios
    assert!(resumen.2 >= 2); // total_ordenes

    // Top Vendedores
    let top_vend_msg = reports_call.top_vendedores(5);
    let result = client.call(&ink_e2e::alice(), &top_vend_msg).submit().await.expect("top_vend failed");
    let top_vend: Vec<UsuarioConReputacion> = result.return_value();

    assert!(!top_vend.is_empty());
    // Alice debe estar ahí

    // Productos más vendidos
    let mas_vendidos_msg = reports_call.productos_mas_vendidos(5);
    let result = client.call(&ink_e2e::alice(), &mas_vendidos_msg).submit().await.expect("mas_vendidos failed");
    let mas_vendidos: Vec<ProductoVendido> = result.return_value();

    assert!(!mas_vendidos.is_empty());

    // Estadísticas por categoría
    let stats_cat_msg = reports_call.estadisticas_categoria(String::from("Tech"));
    let result = client.call(&ink_e2e::alice(), &stats_cat_msg).submit().await.expect("stats_cat failed");
    let stats_cat: Result<EstadisticasCategoria, ReportError> = result.return_value();
    
    assert!(stats_cat.is_ok());

    Ok(())
}
