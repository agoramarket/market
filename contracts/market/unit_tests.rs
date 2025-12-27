use ink::env::{test, DefaultEnvironment};

mod tests {
    use super::*;

    fn set_next_caller(caller: AccountId) {
        test::set_caller::<DefaultEnvironment>(caller);
    }

    fn get_accounts() -> test::DefaultAccounts<DefaultEnvironment> {
        test::default_accounts::<DefaultEnvironment>()
    }

    fn set_value(amount: Balance) {
        test::set_value_transferred::<DefaultEnvironment>(amount);
    }

    fn setup_vendedor() -> (test::DefaultAccounts<DefaultEnvironment>, Marketplace) {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();
        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        (accounts, mp)
    }

    fn setup_con_producto() -> (test::DefaultAccounts<DefaultEnvironment>, Marketplace, u32) {
        let (accounts, mut mp) = setup_vendedor();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();
        (accounts, mp, pid)
    }

    fn setup_vendedor_producto_comprador() -> (test::DefaultAccounts<DefaultEnvironment>, Marketplace, u32) {
        let (accounts, mut mp, pid) = setup_con_producto();
        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        (accounts, mp, pid)
    }

    fn setup_con_orden(cantidad: u32, precio: Balance) -> (test::DefaultAccounts<DefaultEnvironment>, Marketplace, u32, u32) {
        let (accounts, mut mp, pid) = setup_vendedor_producto_comprador();
        set_next_caller(accounts.bob);
        set_value(precio * cantidad as u128);
        let oid = mp.comprar(pid, cantidad).unwrap();
        (accounts, mp, pid, oid)
    }

    fn setup_orden_enviada() -> (test::DefaultAccounts<DefaultEnvironment>, Marketplace, u32, u32) {
        let (accounts, mut mp, pid, oid) = setup_con_orden(1, 100);
        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();
        (accounts, mp, pid, oid)
    }

    fn setup_orden_recibida() -> (test::DefaultAccounts<DefaultEnvironment>, Marketplace, u32, u32) {
        let (accounts, mut mp, pid, oid) = setup_orden_enviada();
        set_next_caller(accounts.bob);
        let _ = mp.marcar_recibido(oid);
        (accounts, mp, pid, oid)
    }

    fn setup_orden_cancelada() -> (test::DefaultAccounts<DefaultEnvironment>, Marketplace, u32, u32) {
        let (accounts, mut mp, pid, oid) = setup_con_orden(1, 100);
        set_next_caller(accounts.bob);
        mp.solicitar_cancelacion(oid).unwrap();
        set_next_caller(accounts.alice);
        let _ = mp.aceptar_cancelacion(oid);
        (accounts, mp, pid, oid)
    }

    #[ink::test]
    fn registro_todos_roles() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        assert_eq!(mp.registrar(Rol::Comprador), Ok(()));
        assert_eq!(mp.obtener_rol(accounts.alice), Some(Rol::Comprador));

        set_next_caller(accounts.bob);
        assert_eq!(mp.registrar(Rol::Vendedor), Ok(()));
        assert_eq!(mp.obtener_rol(accounts.bob), Some(Rol::Vendedor));

        set_next_caller(accounts.charlie);
        assert_eq!(mp.registrar(Rol::Ambos), Ok(()));
        assert_eq!(mp.obtener_rol(accounts.charlie), Some(Rol::Ambos));

        set_next_caller(accounts.alice);
        assert_eq!(mp.registrar(Rol::Vendedor), Err(Error::YaRegistrado));
    }

    #[ink::test]
    fn modificar_rol_casos() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        assert_eq!(mp.modificar_rol(Rol::Ambos), Err(Error::SinRegistro));

        mp.registrar(Rol::Comprador).unwrap();
        assert_eq!(mp.modificar_rol(Rol::Ambos), Ok(()));
        assert_eq!(mp.obtener_rol(accounts.alice), Some(Rol::Ambos));

        assert_eq!(mp.modificar_rol(Rol::Ambos), Ok(()));

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Vendedor).unwrap();
        assert_eq!(mp.modificar_rol(Rol::Ambos), Ok(()));
        assert_eq!(mp.obtener_rol(accounts.bob), Some(Rol::Ambos));
    }

    #[ink::test]
    fn publicar_producto_exitoso() {
        let (accounts, mut mp) = setup_vendedor();

        let pid = mp
            .publicar(
                "Laptop".to_string(),
                "Laptop gaming de alta gama".to_string(),
                1500,
                5,
                "Electrónica".to_string(),
            )
            .unwrap();

        let producto = mp.obtener_producto(pid).unwrap();
        assert_eq!(producto.vendedor, accounts.alice);
        assert_eq!(producto.nombre, "Laptop");
        assert_eq!(producto.descripcion, "Laptop gaming de alta gama");
        assert_eq!(producto.precio, 1500);
        assert_eq!(producto.stock, 5);
        assert_eq!(producto.categoria, "Electrónica");
    }

    #[ink::test]
    fn publicar_producto_errores() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        assert_eq!(
            mp.publicar("Test".to_string(), "Desc".to_string(), 100, 5, "Cat".to_string()),
            Err(Error::SinRegistro)
        );

        mp.registrar(Rol::Comprador).unwrap();
        assert_eq!(
            mp.publicar("Test".to_string(), "Desc".to_string(), 100, 5, "Cat".to_string()),
            Err(Error::SinPermiso)
        );

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Vendedor).unwrap();

        assert_eq!(
            mp.publicar("Test".to_string(), "Desc".to_string(), 0, 5, "Cat".to_string()),
            Err(Error::ParamInvalido)
        );

        assert_eq!(
            mp.publicar("Test".to_string(), "Desc".to_string(), 100, 0, "Cat".to_string()),
            Err(Error::ParamInvalido)
        );

        assert_eq!(
            mp.publicar("".to_string(), "Desc".to_string(), 100, 5, "Cat".to_string()),
            Err(Error::ParamInvalido)
        );

        assert_eq!(
            mp.publicar("Test".to_string(), "".to_string(), 100, 5, "Cat".to_string()),
            Err(Error::ParamInvalido)
        );

        assert_eq!(
            mp.publicar("Test".to_string(), "Desc".to_string(), 100, 5, "".to_string()),
            Err(Error::ParamInvalido)
        );

        assert_eq!(
            mp.publicar("a".repeat(65), "Desc".to_string(), 100, 5, "Cat".to_string()),
            Err(Error::ParamInvalido)
        );

        assert_eq!(
            mp.publicar("Test".to_string(), "a".repeat(257), 100, 5, "Cat".to_string()),
            Err(Error::ParamInvalido)
        );

        assert_eq!(
            mp.publicar("Test".to_string(), "Desc".to_string(), 100, 5, "a".repeat(33)),
            Err(Error::ParamInvalido)
        );

        assert!(mp.publicar("A".repeat(64), "Desc".to_string(), 100, 10, "Cat".to_string()).is_ok());
    }

    #[ink::test]
    fn listar_productos_de_vendedor() {
        let (accounts, mut mp) = setup_vendedor();

        assert!(mp.listar_productos_de_vendedor(accounts.alice).is_empty());

        mp.publicar("Producto1".to_string(), "Desc1".to_string(), 100, 5, "Cat1".to_string()).unwrap();
        mp.publicar("Producto2".to_string(), "Desc2".to_string(), 200, 10, "Cat2".to_string()).unwrap();

        let productos = mp.listar_productos_de_vendedor(accounts.alice);
        assert_eq!(productos.len(), 2);
        assert_eq!(productos[0].nombre, "Producto1");
        assert_eq!(productos[1].nombre, "Producto2");
    }

    #[ink::test]
    fn comprar_producto_exitoso() {
        let (accounts, mut mp, pid) = setup_vendedor_producto_comprador();

        set_next_caller(accounts.bob);
        set_value(300);
        let oid = mp.comprar(pid, 3).unwrap();

        assert_eq!(oid, 1);
        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 7);

        let orden = mp.obtener_orden(oid).unwrap();
        assert_eq!(orden.comprador, accounts.bob);
        assert_eq!(orden.vendedor, accounts.alice);
        assert_eq!(orden.cantidad, 3);
        assert_eq!(orden.estado, Estado::Pendiente);
        assert_eq!(orden.monto_total, 300);
    }

    #[ink::test]
    fn comprar_errores() {
        let (accounts, mut mp, pid) = setup_con_producto();

        set_next_caller(accounts.bob);
        set_value(100);
        assert_eq!(mp.comprar(pid, 1), Err(Error::SinRegistro));

        set_next_caller(accounts.charlie);
        mp.registrar(Rol::Vendedor).unwrap();
        set_value(100);
        assert_eq!(mp.comprar(pid, 1), Err(Error::SinPermiso));

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();

        set_value(0);
        assert_eq!(mp.comprar(pid, 0), Err(Error::ParamInvalido));

        set_value(100);
        assert_eq!(mp.comprar(999, 1), Err(Error::ProdInexistente));

        set_value(1100);
        assert_eq!(mp.comprar(pid, 11), Err(Error::StockInsuf));

        set_value(50);
        assert_eq!(mp.comprar(pid, 1), Err(Error::PagoInsuficiente));

        set_value(150);
        assert_eq!(mp.comprar(pid, 1), Err(Error::PagoExcesivo));
    }

    #[ink::test]
    fn auto_compra_prohibida() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Ambos).unwrap();
        let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

        set_value(100);
        assert_eq!(mp.comprar(pid, 1), Err(Error::AutoCompraProhibida));
    }

    #[ink::test]
    fn comprar_todo_el_stock() {
        let (accounts, mut mp, pid) = setup_vendedor_producto_comprador();

        set_next_caller(accounts.bob);
        set_value(1000);
        assert!(mp.comprar(pid, 10).is_ok());

        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 0);

        set_value(100);
        assert_eq!(mp.comprar(pid, 1), Err(Error::StockInsuf));
    }

    #[ink::test]
    fn rol_ambos_puede_comprar_a_otros() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Ambos).unwrap();
        mp.publicar("Test Alice".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Ambos).unwrap();
        let pid_bob = mp.publicar("Test Bob".to_string(), "Desc".to_string(), 50, 5, "Cat".to_string()).unwrap();

        set_next_caller(accounts.alice);
        set_value(100);
        let oid = mp.comprar(pid_bob, 2).unwrap();
        assert_eq!(oid, 1);
        assert_eq!(mp.obtener_producto(pid_bob).unwrap().stock, 3);
    }

    #[ink::test]
    fn listar_ordenes_de_comprador() {
        let (accounts, mut mp, pid) = setup_vendedor_producto_comprador();

        set_next_caller(accounts.bob);
        assert!(mp.listar_ordenes_de_comprador(accounts.bob).is_empty());

        set_value(200);
        mp.comprar(pid, 2).unwrap();
        set_value(300);
        mp.comprar(pid, 3).unwrap();

        let ordenes = mp.listar_ordenes_de_comprador(accounts.bob);
        assert_eq!(ordenes.len(), 2);
        assert_eq!(ordenes[0].cantidad, 2);
        assert_eq!(ordenes[1].cantidad, 3);
    }

    #[ink::test]
    fn multiples_ordenes_mismo_producto() {
        let (accounts, mut mp, pid) = setup_vendedor_producto_comprador();

        set_next_caller(accounts.bob);
        set_value(300);
        mp.comprar(pid, 3).unwrap();
        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 7);

        set_next_caller(accounts.charlie);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(400);
        mp.comprar(pid, 4).unwrap();
        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 3);
    }

    #[ink::test]
    fn flujo_orden_completo() {
        let (accounts, mut mp, _, oid) = setup_con_orden(1, 100);

        set_next_caller(accounts.alice);
        assert_eq!(mp.marcar_enviado(oid), Ok(()));
        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Enviado);

        set_next_caller(accounts.bob);
        assert_eq!(mp.marcar_recibido(oid), Ok(()));
        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Recibido);
    }

    #[ink::test]
    fn marcar_enviado_errores() {
        let (accounts, mut mp, _, oid) = setup_con_orden(1, 100);

        set_next_caller(accounts.bob);
        assert_eq!(mp.marcar_enviado(oid), Err(Error::SinPermiso));

        set_next_caller(accounts.charlie);
        mp.registrar(Rol::Vendedor).unwrap();
        assert_eq!(mp.marcar_enviado(oid), Err(Error::SinPermiso));

        set_next_caller(accounts.alice);
        assert_eq!(mp.marcar_enviado(999), Err(Error::OrdenInexistente));

        mp.marcar_enviado(oid).unwrap();
        assert_eq!(mp.marcar_enviado(oid), Err(Error::EstadoInvalido));
    }

    #[ink::test]
    fn marcar_recibido_errores() {
        let (accounts, mut mp, _, oid) = setup_con_orden(1, 100);

        set_next_caller(accounts.bob);
        assert_eq!(mp.marcar_recibido(oid), Err(Error::EstadoInvalido));

        assert_eq!(mp.marcar_recibido(999), Err(Error::OrdenInexistente));

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        assert_eq!(mp.marcar_recibido(oid), Err(Error::SinPermiso));
    }

    #[ink::test]
    fn marcar_orden_cancelada_falla() {
        let (accounts, mut mp, _, oid) = setup_orden_cancelada();

        set_next_caller(accounts.alice);
        assert_eq!(mp.marcar_enviado(oid), Err(Error::OrdenCancelada));

        set_next_caller(accounts.bob);
        assert_eq!(mp.marcar_recibido(oid), Err(Error::OrdenCancelada));
    }

    #[ink::test]
    fn obtener_orden_permisos() {
        let (accounts, mut mp, _, oid) = setup_con_orden(1, 100);

        set_next_caller(accounts.bob);
        assert!(mp.obtener_orden(oid).is_ok());

        set_next_caller(accounts.alice);
        assert!(mp.obtener_orden(oid).is_ok());

        set_next_caller(accounts.charlie);
        assert_eq!(mp.obtener_orden(oid), Err(Error::SinPermiso));

        set_next_caller(accounts.alice);
        assert_eq!(mp.obtener_orden(0), Err(Error::OrdenInexistente));
    }

    #[ink::test]
    fn cancelacion_flujo_completo() {
        let (accounts, mut mp, pid, oid) = setup_con_orden(3, 100);

        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 7);

        set_next_caller(accounts.bob);
        assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));

        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Pendiente);

        set_next_caller(accounts.alice);
        assert_eq!(mp.aceptar_cancelacion(oid), Ok(()));

        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Cancelada);
        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 10);

        assert_eq!(mp.rechazar_cancelacion(oid), Err(Error::CancelacionInexistente));
    }

    #[ink::test]
    fn cancelacion_rechazar() {
        let (accounts, mut mp, pid, oid) = setup_con_orden(3, 100);

        set_next_caller(accounts.bob);
        mp.solicitar_cancelacion(oid).unwrap();

        set_next_caller(accounts.alice);
        assert_eq!(mp.rechazar_cancelacion(oid), Ok(()));

        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Pendiente);
        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 7);

        assert_eq!(mp.rechazar_cancelacion(oid), Err(Error::CancelacionInexistente));
    }

    #[ink::test]
    fn cancelacion_desde_vendedor() {
        let (accounts, mut mp, _, oid) = setup_con_orden(3, 100);

        set_next_caller(accounts.alice);
        assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));

        set_next_caller(accounts.bob);
        assert_eq!(mp.aceptar_cancelacion(oid), Ok(()));
        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Cancelada);
    }

    #[ink::test]
    fn cancelacion_en_estado_enviado() {
        let (accounts, mut mp, pid, oid) = setup_orden_enviada();

        set_next_caller(accounts.bob);
        assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));

        set_next_caller(accounts.alice);
        assert_eq!(mp.aceptar_cancelacion(oid), Ok(()));

        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Cancelada);
        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 10);
    }

    #[ink::test]
    fn cancelacion_errores() {
        let (accounts, mut mp, _, oid) = setup_con_orden(1, 100);

        set_next_caller(accounts.alice);
        assert_eq!(mp.solicitar_cancelacion(999), Err(Error::OrdenInexistente));

        assert_eq!(mp.aceptar_cancelacion(oid), Err(Error::CancelacionInexistente));
        assert_eq!(mp.rechazar_cancelacion(oid), Err(Error::CancelacionInexistente));
        assert_eq!(mp.rechazar_cancelacion(999), Err(Error::CancelacionInexistente));

        set_next_caller(accounts.charlie);
        mp.registrar(Rol::Comprador).unwrap();
        assert_eq!(mp.solicitar_cancelacion(oid), Err(Error::SinPermiso));

        set_next_caller(accounts.bob);
        mp.solicitar_cancelacion(oid).unwrap();

        set_next_caller(accounts.charlie);
        assert_eq!(mp.aceptar_cancelacion(oid), Err(Error::SinPermiso));
        assert_eq!(mp.rechazar_cancelacion(oid), Err(Error::SinPermiso));
    }

    #[ink::test]
    fn cancelacion_ya_pendiente() {
        let (accounts, mut mp, _, oid) = setup_con_orden(1, 100);

        set_next_caller(accounts.bob);
        mp.solicitar_cancelacion(oid).unwrap();

        set_next_caller(accounts.alice);
        assert_eq!(mp.solicitar_cancelacion(oid), Err(Error::CancelacionYaPendiente));
    }

    #[ink::test]
    fn cancelacion_solicitante_no_puede_aceptar_ni_rechazar() {
        let (accounts, mut mp, _, oid) = setup_con_orden(1, 100);

        set_next_caller(accounts.bob);
        mp.solicitar_cancelacion(oid).unwrap();

        assert_eq!(mp.aceptar_cancelacion(oid), Err(Error::SolicitanteCancelacion));
        assert_eq!(mp.rechazar_cancelacion(oid), Err(Error::SolicitanteCancelacion));
    }

    #[ink::test]
    fn cancelacion_orden_ya_cancelada() {
        let (accounts, mut mp, _, oid) = setup_orden_cancelada();

        set_next_caller(accounts.bob);
        assert_eq!(mp.solicitar_cancelacion(oid), Err(Error::OrdenCancelada));
    }

    #[ink::test]
    fn cancelacion_orden_recibida() {
        let (accounts, mut mp, _, oid) = setup_orden_recibida();

        set_next_caller(accounts.bob);
        assert_eq!(mp.solicitar_cancelacion(oid), Err(Error::EstadoInvalido));
    }

    #[ink::test]
    fn resolicitar_cancelacion_despues_de_rechazo() {
        let (accounts, mut mp, _, oid) = setup_con_orden(1, 100);

        set_next_caller(accounts.bob);
        mp.solicitar_cancelacion(oid).unwrap();

        set_next_caller(accounts.alice);
        mp.rechazar_cancelacion(oid).unwrap();

        set_next_caller(accounts.bob);
        assert!(mp.solicitar_cancelacion(oid).is_ok());
    }

    #[ink::test]
    fn cancelacion_overflow_stock() {
        let (accounts, mut mp, pid, oid) = setup_con_orden(1, 100);

        let mut prod = mp.obtener_producto(pid).unwrap();
        prod.stock = u32::MAX;
        mp.productos.insert(pid, &prod);

        set_next_caller(accounts.bob);
        mp.solicitar_cancelacion(oid).unwrap();

        set_next_caller(accounts.alice);
        assert_eq!(mp.aceptar_cancelacion(oid), Err(Error::StockOverflow));
    }

    #[ink::test]
    fn calificar_vendedor_exitoso() {
        let (accounts, mut mp, _, oid) = setup_orden_recibida();

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Ok(()));

        let rep = mp.obtener_reputacion(accounts.alice).unwrap();
        assert_eq!(rep.como_vendedor, (5, 1));
    }

    #[ink::test]
    fn calificar_comprador_exitoso() {
        let (accounts, mut mp, _, oid) = setup_orden_recibida();

        set_next_caller(accounts.alice);
        assert_eq!(mp.calificar_comprador(oid, 4), Ok(()));

        let rep = mp.obtener_reputacion(accounts.bob).unwrap();
        assert_eq!(rep.como_comprador, (4, 1));
    }

    #[ink::test]
    fn calificacion_bidireccional() {
        let (accounts, mut mp, _, oid) = setup_orden_recibida();

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Ok(()));

        set_next_caller(accounts.alice);
        assert_eq!(mp.calificar_comprador(oid, 4), Ok(()));

        assert_eq!(mp.obtener_reputacion(accounts.alice).unwrap().como_vendedor, (5, 1));
        assert_eq!(mp.obtener_reputacion(accounts.bob).unwrap().como_comprador, (4, 1));
    }

    #[ink::test]
    fn calificar_errores_permisos() {
        let (accounts, mut mp, _, oid) = setup_orden_recibida();

        set_next_caller(accounts.charlie);
        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::SinPermiso));
        assert_eq!(mp.calificar_comprador(oid, 4), Err(Error::SinPermiso));

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(999, 5), Err(Error::OrdenInexistente));
        set_next_caller(accounts.alice);
        assert_eq!(mp.calificar_comprador(999, 4), Err(Error::OrdenInexistente));
    }

    #[ink::test]
    fn calificar_puntos_invalidos() {
        let (accounts, mut mp, _, oid) = setup_orden_recibida();

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 0), Err(Error::CalificacionInvalida));
        assert_eq!(mp.calificar_vendedor(oid, 6), Err(Error::CalificacionInvalida));

        set_next_caller(accounts.alice);
        assert_eq!(mp.calificar_comprador(oid, 0), Err(Error::CalificacionInvalida));
        assert_eq!(mp.calificar_comprador(oid, 6), Err(Error::CalificacionInvalida));
    }

    #[ink::test]
    fn calificar_dos_veces() {
        let (accounts, mut mp, _, oid) = setup_orden_recibida();

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Ok(()));
        assert_eq!(mp.calificar_vendedor(oid, 4), Err(Error::YaCalificado));

        set_next_caller(accounts.alice);
        assert_eq!(mp.calificar_comprador(oid, 5), Ok(()));
        assert_eq!(mp.calificar_comprador(oid, 4), Err(Error::YaCalificado));
    }

    #[ink::test]
    fn calificar_orden_no_recibida() {
        let (accounts, mut mp, _, oid) = setup_con_orden(1, 100);

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::OrdenNoRecibida));

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();
        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::OrdenNoRecibida));
    }

    #[ink::test]
    fn calificar_orden_cancelada() {
        let (accounts, mut mp, _, oid) = setup_orden_cancelada();

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::OrdenNoRecibida));
    }

    #[ink::test]
    fn calificaciones_multiples_acumulan() {
        let (accounts, mut mp) = setup_vendedor();

        let pid1 = mp.publicar("Test1".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();
        let pid2 = mp.publicar("Test2".to_string(), "Desc".to_string(), 200, 10, "Cat".to_string()).unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();

        set_value(100);
        let oid1 = mp.comprar(pid1, 1).unwrap();
        set_value(200);
        let oid2 = mp.comprar(pid2, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid1).unwrap();
        mp.marcar_enviado(oid2).unwrap();

        set_next_caller(accounts.bob);
        let _ = mp.marcar_recibido(oid1);
        let _ = mp.marcar_recibido(oid2);

        mp.calificar_vendedor(oid1, 5).unwrap();
        mp.calificar_vendedor(oid2, 3).unwrap();

        let rep = mp.obtener_reputacion(accounts.alice).unwrap();
        assert_eq!(rep.como_vendedor, (8, 2));

        let cat = mp.obtener_calificacion_categoria("Cat".to_string()).unwrap();
        assert_eq!(cat, (8, 2));
    }

    #[ink::test]
    fn reputacion_sin_calificaciones() {
        let accounts = get_accounts();
        let mp = Marketplace::new();
        assert_eq!(mp.obtener_reputacion(accounts.alice), None);
    }

    #[ink::test]
    fn overflow_reputacion() {
        let (accounts, mut mp, _, oid) = setup_orden_recibida();

        let mut rep = mp.reputaciones.get(accounts.alice).unwrap_or(ReputacionUsuario {
            como_comprador: (0, 0),
            como_vendedor: (u32::MAX - 2, 1),
        });
        rep.como_vendedor = (u32::MAX - 2, 1);
        mp.reputaciones.insert(accounts.alice, &rep);

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::OverflowAritmetico));
    }

    #[ink::test]
    fn overflow_cantidad_calificaciones() {
        let (accounts, mut mp, _, oid) = setup_orden_recibida();

        let mut rep = mp.reputaciones.get(accounts.alice).unwrap_or(ReputacionUsuario {
            como_comprador: (0, 0),
            como_vendedor: (10, u32::MAX),
        });
        rep.como_vendedor = (10, u32::MAX);
        mp.reputaciones.insert(accounts.alice, &rep);

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::OverflowAritmetico));
    }

    #[ink::test]
    fn fondos_retenidos_y_liberados() {
        let (accounts, mut mp, _, oid) = setup_con_orden(3, 100);

        assert_eq!(mp.obtener_fondos_retenidos(oid), 300);

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        let _ = mp.marcar_recibido(oid);

        assert_eq!(mp.obtener_fondos_retenidos(oid), 0);
    }

    #[ink::test]
    fn fondos_devueltos_al_cancelar() {
        let (accounts, mut mp, _, oid) = setup_con_orden(2, 100);

        assert_eq!(mp.obtener_fondos_retenidos(oid), 200);

        set_next_caller(accounts.bob);
        mp.solicitar_cancelacion(oid).unwrap();

        set_next_caller(accounts.alice);
        let _ = mp.aceptar_cancelacion(oid);

        assert_eq!(mp.obtener_fondos_retenidos(oid), 0);
    }

    #[ink::test]
    fn monto_total_en_orden() {
        let (accounts, mut mp) = setup_vendedor();
        let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 50, 10, "Cat".to_string()).unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(250);
        let oid = mp.comprar(pid, 5).unwrap();

        let orden = mp.obtener_orden(oid).unwrap();
        assert_eq!(orden.monto_total, 250);
    }

    #[ink::test]
    fn overflow_id_producto() {
        let (_, mut mp) = setup_vendedor();

        mp.next_prod_id = u32::MAX;
        assert_eq!(
            mp.publicar("Test".to_string(), "Desc".to_string(), 100, 5, "Cat".to_string()),
            Err(Error::IdOverflow)
        );
    }

    #[ink::test]
    fn overflow_id_orden() {
        let (accounts, mut mp, pid) = setup_vendedor_producto_comprador();

        mp.next_order_id = u32::MAX;
        set_next_caller(accounts.bob);
        set_value(100);
        assert_eq!(mp.comprar(pid, 1), Err(Error::IdOverflow));
    }

    #[ink::test]
    fn get_total_productos() {
        let (_, mut mp) = setup_vendedor();

        assert_eq!(mp.get_total_productos(), 0);
        mp.publicar("P1".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();
        assert_eq!(mp.get_total_productos(), 1);
        mp.publicar("P2".to_string(), "Desc".to_string(), 200, 5, "Cat".to_string()).unwrap();
        assert_eq!(mp.get_total_productos(), 2);
    }

    #[ink::test]
    fn get_total_ordenes() {
        let (accounts, mut mp, pid) = setup_vendedor_producto_comprador();

        assert_eq!(mp.get_total_ordenes(), 0);
        set_next_caller(accounts.bob);
        set_value(100);
        mp.comprar(pid, 1).unwrap();
        assert_eq!(mp.get_total_ordenes(), 1);
        set_value(200);
        mp.comprar(pid, 2).unwrap();
        assert_eq!(mp.get_total_ordenes(), 2);
    }

    #[ink::test]
    fn obtener_producto_y_orden_inexistentes() {
        let accounts = get_accounts();
        let mp = Marketplace::new();

        assert!(mp.obtener_producto(0).is_none());
        assert!(mp.obtener_producto(999).is_none());

        set_next_caller(accounts.alice);
        assert_eq!(mp.obtener_orden(0), Err(Error::OrdenInexistente));
    }

    #[ink::test]
    fn obtener_orden_publica() {
        let (accounts, mp, _, oid) = setup_con_orden(1, 100);

        let orden = mp.obtener_orden_publica(oid).unwrap();
        assert_eq!(orden.comprador, accounts.bob);
        assert_eq!(orden.vendedor, accounts.alice);
        assert_eq!(orden.cantidad, 1);

        assert!(mp.obtener_orden_publica(999).is_none());
    }

    #[ink::test]
    fn listar_usuarios() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        assert!(mp.listar_usuarios().is_empty());

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        assert_eq!(mp.listar_usuarios().len(), 1);

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        assert_eq!(mp.listar_usuarios().len(), 2);
    }

    #[ink::test]
    fn listar_todos_productos() {
        let (_, mut mp) = setup_vendedor();

        assert!(mp.listar_todos_productos().is_empty());

        let pid1 = mp.publicar("Prod1".to_string(), "Desc1".to_string(), 100, 10, "Cat1".to_string()).unwrap();
        let pid2 = mp.publicar("Prod2".to_string(), "Desc2".to_string(), 200, 5, "Cat2".to_string()).unwrap();

        let productos = mp.listar_todos_productos();
        assert_eq!(productos.len(), 2);
        assert_eq!(productos[0].0, pid1);
        assert_eq!(productos[0].1.nombre, "Prod1");
        assert_eq!(productos[1].0, pid2);
        assert_eq!(productos[1].1.nombre, "Prod2");
    }

    #[ink::test]
    fn listar_todas_ordenes() {
        let (accounts, mut mp) = setup_vendedor();
        let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 100, "Cat".to_string()).unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();

        assert!(mp.listar_todas_ordenes().is_empty());

        set_value(100);
        let oid1 = mp.comprar(pid, 1).unwrap();
        set_value(200);
        let oid2 = mp.comprar(pid, 2).unwrap();

        let ordenes = mp.listar_todas_ordenes();
        assert_eq!(ordenes.len(), 2);
        assert_eq!(ordenes[0].0, oid1);
        assert_eq!(ordenes[0].1.cantidad, 1);
        assert_eq!(ordenes[1].0, oid2);
        assert_eq!(ordenes[1].1.cantidad, 2);
    }

    #[ink::test]
    fn listar_todas_reputaciones() {
        let (accounts, mut mp, _, oid) = setup_orden_recibida();

        assert!(mp.listar_todas_reputaciones().is_empty());

        set_next_caller(accounts.bob);
        mp.calificar_vendedor(oid, 5).unwrap();

        set_next_caller(accounts.alice);
        mp.calificar_comprador(oid, 4).unwrap();

        let reputaciones = mp.listar_todas_reputaciones();
        assert_eq!(reputaciones.len(), 2);
    }
}