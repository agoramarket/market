use ink::env::{test, DefaultEnvironment};

mod tests {
    use super::*;

    fn set_next_caller(caller: AccountId) {
        test::set_caller::<DefaultEnvironment>(caller);
    }

    fn get_accounts() -> test::DefaultAccounts<DefaultEnvironment> {
        test::default_accounts::<DefaultEnvironment>()
    }

    /// Helper para establecer el valor transferido en tests (simula pago).
    fn set_value(amount: Balance) {
        test::set_value_transferred::<DefaultEnvironment>(amount);
    }
    /// Test: Registro exitoso de usuario con rol Comprador.
    #[ink::test]
    fn registro_comprador_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        assert_eq!(mp.registrar(Rol::Comprador), Ok(()));
        assert_eq!(mp.obtener_rol(accounts.alice), Some(Rol::Comprador));
    }

    /// Test: Registro exitoso de usuario con rol Vendedor.
    #[ink::test]
    fn registro_vendedor_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.bob);
        assert_eq!(mp.registrar(Rol::Vendedor), Ok(()));
        assert_eq!(mp.obtener_rol(accounts.bob), Some(Rol::Vendedor));
    }

    /// Test: Registro exitoso de usuario con rol Ambos.
    #[ink::test]
    fn registro_ambos_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.charlie);
        assert_eq!(mp.registrar(Rol::Ambos), Ok(()));
        assert_eq!(mp.obtener_rol(accounts.charlie), Some(Rol::Ambos));
    }

    /// Test: Error al intentar registrar un usuario ya registrado.
    #[ink::test]
    fn registro_usuario_ya_registrado() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Comprador).unwrap();
        assert_eq!(mp.registrar(Rol::Vendedor), Err(Error::YaRegistrado));
    }

    /// Test: Modificación exitosa de rol de Comprador a Ambos.
    #[ink::test]
    fn modificar_rol_comprador_a_ambos() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Comprador).unwrap();
        assert_eq!(mp.obtener_rol(accounts.alice), Some(Rol::Comprador));

        assert_eq!(mp.modificar_rol(Rol::Ambos), Ok(()));
        assert_eq!(mp.obtener_rol(accounts.alice), Some(Rol::Ambos));
    }

    /// Test: Modificación exitosa de rol de Vendedor a Ambos.
    #[ink::test]
    fn modificar_rol_vendedor_a_ambos() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Vendedor).unwrap();
        assert_eq!(mp.obtener_rol(accounts.bob), Some(Rol::Vendedor));

        assert_eq!(mp.modificar_rol(Rol::Ambos), Ok(()));
        assert_eq!(mp.obtener_rol(accounts.bob), Some(Rol::Ambos));
    }

    /// Test: Error al intentar modificar rol sin estar registrado.
    #[ink::test]
    fn modificar_rol_sin_registro() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        assert_eq!(mp.modificar_rol(Rol::Ambos), Err(Error::SinRegistro));
    }

    /// Test: Publicación exitosa de producto por vendedor.
    #[ink::test]
    fn publicar_producto_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        let resultado = mp.publicar(
            "Laptop".to_string(),
            "Laptop gaming de alta gama".to_string(),
            1500,
            5,
            "Electrónica".to_string(),
        );
        assert_eq!(resultado, Ok(1));

        let producto = mp.obtener_producto(1).unwrap();
        assert_eq!(producto.vendedor, accounts.alice);
        assert_eq!(producto.nombre, "Laptop");
        assert_eq!(producto.descripcion, "Laptop gaming de alta gama");
        assert_eq!(producto.precio, 1500);
        assert_eq!(producto.stock, 5);
        assert_eq!(producto.categoria, "Electrónica");
    }

    /// Test: Error al publicar producto sin ser vendedor.
    #[ink::test]
    fn publicar_producto_sin_permiso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Comprador).unwrap();

        let resultado = mp.publicar(
            "Test".to_string(),
            "Desc".to_string(),
            100,
            5,
            "Cat".to_string(),
        );
        assert_eq!(resultado, Err(Error::SinPermiso));
    }

    /// Test: Error al publicar producto sin estar registrado.
    #[ink::test]
    fn publicar_producto_sin_registro() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        let resultado = mp.publicar(
            "Test".to_string(),
            "Desc".to_string(),
            100,
            5,
            "Cat".to_string(),
        );
        assert_eq!(resultado, Err(Error::SinRegistro));
    }

    /// Test: Error al publicar producto con precio cero.
    #[ink::test]
    fn publicar_producto_precio_invalido() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        let resultado = mp.publicar(
            "Test".to_string(),
            "Desc".to_string(),
            0,
            5,
            "Cat".to_string(),
        );
        assert_eq!(resultado, Err(Error::ParamInvalido));
    }

    /// Test: Error al publicar producto con stock cero.
    #[ink::test]
    fn publicar_producto_stock_invalido() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        let resultado = mp.publicar(
            "Test".to_string(),
            "Desc".to_string(),
            100,
            0,
            "Cat".to_string(),
        );
        assert_eq!(resultado, Err(Error::ParamInvalido));
    }

    /// Test: Error al publicar producto con nombre muy largo.
    #[ink::test]
    fn publicar_producto_nombre_muy_largo() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        let nombre_largo = "a".repeat(65);
        let resultado = mp.publicar(nombre_largo, "Desc".to_string(), 100, 5, "Cat".to_string());
        assert_eq!(resultado, Err(Error::ParamInvalido));
    }

    /// Test: Error al publicar producto con descripción muy larga.
    #[ink::test]
    fn publicar_producto_descripcion_muy_larga() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        let descripcion_larga = "a".repeat(257);
        let resultado = mp.publicar(
            "Test".to_string(),
            descripcion_larga,
            100,
            5,
            "Cat".to_string(),
        );
        assert_eq!(resultado, Err(Error::ParamInvalido));
    }

    /// Test: Error al publicar producto con categoría muy larga.
    #[ink::test]
    fn publicar_producto_categoria_muy_larga() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        let categoria_larga = "a".repeat(33);
        let resultado = mp.publicar(
            "Test".to_string(),
            "Desc".to_string(),
            100,
            5,
            categoria_larga,
        );
        assert_eq!(resultado, Err(Error::ParamInvalido));
    }

    /// Test: Error al publicar producto con nombre vacío.
    #[ink::test]
    fn publicar_producto_nombre_vacio() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        let resultado = mp.publicar(
            "".to_string(),
            "Descripción válida".to_string(),
            100,
            5,
            "Categoría".to_string(),
        );
        assert_eq!(resultado, Err(Error::ParamInvalido));
    }

    /// Test: Error al publicar producto con descripción vacía.
    #[ink::test]
    fn publicar_producto_descripcion_vacia() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        let resultado = mp.publicar(
            "Producto".to_string(),
            "".to_string(),
            100,
            5,
            "Categoría".to_string(),
        );
        assert_eq!(resultado, Err(Error::ParamInvalido));
    }

    /// Test: Error al publicar producto con categoría vacía.
    #[ink::test]
    fn publicar_producto_categoria_vacia() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        let resultado = mp.publicar(
            "Producto".to_string(),
            "Descripción válida".to_string(),
            100,
            5,
            "".to_string(),
        );
        assert_eq!(resultado, Err(Error::ParamInvalido));
    }

    /// Test: Listar productos de un vendedor.
    #[ink::test]
    fn listar_productos_de_vendedor() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        mp.publicar(
            "Producto1".to_string(),
            "Desc1".to_string(),
            100,
            5,
            "Cat1".to_string(),
        )
        .unwrap();
        mp.publicar(
            "Producto2".to_string(),
            "Desc2".to_string(),
            200,
            10,
            "Cat2".to_string(),
        )
        .unwrap();

        let productos = mp.listar_productos_de_vendedor(accounts.alice);
        assert_eq!(productos.len(), 2);
        assert_eq!(productos[0].nombre, "Producto1");
        assert_eq!(productos[1].nombre, "Producto2");
    }

    /// Test: Listar productos de vendedor sin productos retorna vector vacío.
    #[ink::test]
    fn listar_productos_vendedor_sin_productos() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        let productos = mp.listar_productos_de_vendedor(accounts.alice);
        assert_eq!(productos.len(), 0);
    }

    /// Test: Compra exitosa de producto.
    #[ink::test]
    fn comprar_producto_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(300); // 100 * 3 = 300
        let resultado = mp.comprar(pid, 3);

        assert_eq!(resultado, Ok(1));

        let producto = mp.obtener_producto(pid).unwrap();
        assert_eq!(producto.stock, 7);

        let orden = mp.obtener_orden(1).unwrap();
        assert_eq!(orden.comprador, accounts.bob);
        assert_eq!(orden.vendedor, accounts.alice);
        assert_eq!(orden.cantidad, 3);
        assert_eq!(orden.estado, Estado::Pendiente);
        assert_eq!(orden.monto_total, 300);
    }

    /// Test: Error al comprar sin ser comprador.
    #[ink::test]
    fn comprar_sin_permiso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.charlie);
        mp.registrar(Rol::Vendedor).unwrap();
        set_value(100); // Aún debería fallar por SinPermiso
        let resultado = mp.comprar(pid, 1);
        assert_eq!(resultado, Err(Error::SinPermiso));
    }

    /// Test: Error al intentar auto-comprar su propio producto con rol Ambos.
    #[ink::test]
    fn comprar_auto_producto_vendedor() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Ambos).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_value(100);
        let resultado = mp.comprar(pid, 1);
        assert_eq!(resultado, Err(Error::AutoCompraProhibida));
    }

    /// Test: Error al comprar sin estar registrado.
    #[ink::test]
    fn comprar_sin_registro() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        set_value(100);
        let resultado = mp.comprar(pid, 1);
        assert_eq!(resultado, Err(Error::SinRegistro));
    }

    /// Test: Error al comprar cantidad cero.
    #[ink::test]
    fn comprar_cantidad_invalida() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(0);
        let resultado = mp.comprar(pid, 0);
        assert_eq!(resultado, Err(Error::ParamInvalido));
    }

    /// Test: Error al comprar producto inexistente.
    #[ink::test]
    fn comprar_producto_inexistente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let resultado = mp.comprar(999, 1);
        assert_eq!(resultado, Err(Error::ProdInexistente));
    }

    /// Test: Error al comprar más stock del disponible.
    #[ink::test]
    fn comprar_stock_insuficiente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                5,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(1000); // Monto para 10 unidades
        let resultado = mp.comprar(pid, 10);
        assert_eq!(resultado, Err(Error::StockInsuf));
    }

    /// Test: Listar órdenes del comprador que llama.
    #[ink::test]
    fn listar_ordenes_de_comprador() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(200); // 100 * 2
        mp.comprar(pid, 2).unwrap();
        set_value(300); // 100 * 3
        mp.comprar(pid, 3).unwrap();

        let ordenes = mp.listar_ordenes_de_comprador(accounts.bob);
        assert_eq!(ordenes.len(), 2);
        assert_eq!(ordenes[0].cantidad, 2);
        assert_eq!(ordenes[1].cantidad, 3);
    }

    /// Test: Listar órdenes cuando no se tienen órdenes retorna vector vacío.
    #[ink::test]
    fn listar_ordenes_comprador_sin_ordenes() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Comprador).unwrap();

        let ordenes = mp.listar_ordenes_de_comprador(accounts.alice);
        assert_eq!(ordenes.len(), 0);
    }

    /// Test: Marcar orden como enviada exitosamente.
    #[ink::test]
    fn marcar_orden_enviado_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        assert_eq!(mp.marcar_enviado(oid), Ok(()));
        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Enviado);
    }

    /// Test: Marcar orden como recibida exitosamente.
    #[ink::test]
    fn marcar_orden_recibido_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        assert_eq!(mp.marcar_recibido(oid), Ok(()));
        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Recibido);
    }

    /// Test: Error al marcar como enviado sin ser el vendedor.
    #[ink::test]
    fn marcar_enviado_sin_permiso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        assert_eq!(mp.marcar_enviado(oid), Err(Error::SinPermiso));
    }

    /// Test: Error al marcar como recibido sin ser el comprador.
    #[ink::test]
    fn marcar_recibido_sin_permiso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        assert_eq!(mp.marcar_recibido(oid), Err(Error::SinPermiso));
    }

    /// Test: Error al marcar como recibido sin estar en estado enviado.
    #[ink::test]
    fn marcar_recibido_estado_invalido() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        assert_eq!(mp.marcar_recibido(oid), Err(Error::EstadoInvalido));
    }

    /// Test: Error al marcar como enviado cuando ya está enviado.
    #[ink::test]
    fn marcar_enviado_ya_enviado() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();
        assert_eq!(mp.marcar_enviado(oid), Err(Error::EstadoInvalido));
    }

    /// Test: Error al marcar orden inexistente.
    #[ink::test]
    fn marcar_enviado_orden_inexistente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        assert_eq!(mp.marcar_enviado(999), Err(Error::OrdenInexistente));
    }

    /// Test: Overflow de ID de producto.
    #[ink::test]
    fn overflow_id_producto() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();

        mp.next_prod_id = u32::MAX;
        let resultado = mp.publicar(
            "Test".to_string(),
            "Desc".to_string(),
            100,
            5,
            "Cat".to_string(),
        );
        assert_eq!(resultado, Err(Error::IdOverflow));
    }

    /// Test: Overflow de ID de orden.
    #[ink::test]
    fn overflow_id_orden() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                5,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();

        mp.next_order_id = u32::MAX;
        set_value(100);
        assert_eq!(mp.comprar(pid, 1), Err(Error::IdOverflow));
    }

    /// Test: Usuario con rol Ambos puede comprar productos de otros vendedores.
    #[ink::test]
    fn rol_ambos_puede_comprar_y_vender() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Ambos).unwrap();
        let _pid_alice = mp
            .publicar(
                "Test Alice".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Ambos).unwrap();
        let pid_bob = mp
            .publicar(
                "Test Bob".to_string(),
                "Desc".to_string(),
                50,
                5,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.alice);
        set_value(100); // 50 * 2
        let oid = mp.comprar(pid_bob, 2).unwrap();
        assert_eq!(oid, 1);

        let producto = mp.obtener_producto(pid_bob).unwrap();
        assert_eq!(producto.stock, 3);
    }

    /// Test: Error al auto-comprar con rol Ambos.
    #[ink::test]
    fn comprar_propio_producto_rol_ambos() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Ambos).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_value(100);
        let resultado = mp.comprar(pid, 1);
        assert_eq!(resultado, Err(Error::AutoCompraProhibida));
    }

    /// Test: Error al intentar obtener orden sin ser comprador ni vendedor.
    #[ink::test]
    fn obtener_orden_sin_permiso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.charlie);
        assert_eq!(mp.obtener_orden(oid), Err(Error::SinPermiso));
    }

    /// Test: Solicitar cancelación exitosamente desde el comprador.
    #[ink::test]
    fn solicitar_cancelacion_desde_comprador() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(300); // 100 * 3
        let oid = mp.comprar(pid, 3).unwrap();

        assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));
    }

    /// Test: El comprador solicita cancelación de una orden pendiente (requiere aceptación).
    #[ink::test]
    fn comprador_solicita_cancelacion_pendiente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                5,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(300); // 100 * 3
        let oid = mp.comprar(pid, 3).unwrap();

        // Stock queda en 2 tras la compra.
        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 2);
        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Pendiente);

        // El comprador solicita cancelación.
        assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));

        // La orden sigue pendiente.
        let orden = mp.obtener_orden(oid).unwrap();
        assert_eq!(orden.estado, Estado::Pendiente);

        // Stock sigue en 2.
        let producto = mp.obtener_producto(pid).unwrap();
        assert_eq!(producto.stock, 2);

        // El vendedor acepta la cancelación.
        set_next_caller(accounts.alice);
        assert_eq!(mp.aceptar_cancelacion(oid), Ok(()));

        // Ahora sí está cancelada y stock restaurado.
        let orden = mp.obtener_orden(oid).unwrap();
        assert_eq!(orden.estado, Estado::Cancelada);
        let producto = mp.obtener_producto(pid).unwrap();
        assert_eq!(producto.stock, 5);
    }

    /// Test: Solicitar cancelación exitosamente desde el vendedor.
    #[ink::test]
    fn solicitar_cancelacion_desde_vendedor() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(300); // 100 * 3
        let oid = mp.comprar(pid, 3).unwrap();

        set_next_caller(accounts.alice);
        assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));
    }

    /// Test: Aceptar cancelación desde el otro participante.
    #[ink::test]
    fn aceptar_cancelacion_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(300); // 100 * 3
        let oid = mp.comprar(pid, 3).unwrap();

        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 7);

        assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));

        set_next_caller(accounts.alice);
        assert_eq!(mp.aceptar_cancelacion(oid), Ok(()));

        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Cancelada);

        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 10);

        assert_eq!(
            mp.rechazar_cancelacion(oid),
            Err(Error::CancelacionInexistente)
        );
    }

    /// Test: Rechazar cancelación.
    #[ink::test]
    fn rechazar_cancelacion_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(300); // 100 * 3
        let oid = mp.comprar(pid, 3).unwrap();

        assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));

        set_next_caller(accounts.alice);
        assert_eq!(mp.rechazar_cancelacion(oid), Ok(()));

        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Pendiente);

        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 7);

        assert_eq!(
            mp.rechazar_cancelacion(oid),
            Err(Error::CancelacionInexistente)
        );
    }

    /// Test: Error al solicitar cancelación de orden inexistente.
    #[ink::test]
    fn solicitar_cancelacion_orden_inexistente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Comprador).unwrap();

        assert_eq!(mp.solicitar_cancelacion(999), Err(Error::OrdenInexistente));
    }

    /// Test: Error al solicitar cancelación sin ser participante.
    #[ink::test]
    fn solicitar_cancelacion_sin_permiso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.charlie);
        mp.registrar(Rol::Comprador).unwrap();
        assert_eq!(mp.solicitar_cancelacion(oid), Err(Error::SinPermiso));
    }

    /// Test: Error al solicitar cancelación de orden recibida.
    #[ink::test]
    fn solicitar_cancelacion_orden_recibida() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        mp.marcar_recibido(oid).unwrap();

        assert_eq!(mp.solicitar_cancelacion(oid), Err(Error::EstadoInvalido));
    }

    /// Test: Error al solicitar cancelación de una orden ya cancelada.
    #[ink::test]
    fn solicitar_cancelacion_orden_ya_cancelada() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        mp.solicitar_cancelacion(oid).unwrap();
        set_next_caller(accounts.alice);
        mp.aceptar_cancelacion(oid).unwrap();

        set_next_caller(accounts.bob);
        assert_eq!(mp.solicitar_cancelacion(oid), Err(Error::OrdenCancelada));
    }

    /// Test: El solicitante intenta aceptar su propia cancelación.
    #[ink::test]
    fn solicitante_intenta_aceptar_propia_cancelacion() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        mp.solicitar_cancelacion(oid).unwrap();

        assert_eq!(
            mp.aceptar_cancelacion(oid),
            Err(Error::SolicitanteCancelacion)
        );
    }

    /// Test: El solicitante intenta rechazar su propia cancelación.
    #[ink::test]
    fn solicitante_intenta_rechazar_propia_cancelacion() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        mp.solicitar_cancelacion(oid).unwrap();
        assert_eq!(
            mp.rechazar_cancelacion(oid),
            Err(Error::SolicitanteCancelacion)
        );
    }

    /// Test: Múltiples órdenes del mismo producto por distintos compradores.
    #[ink::test]
    fn multiples_ordenes_mismo_producto() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(300); // 100 * 3
        mp.comprar(pid, 3).unwrap();
        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 7);

        set_next_caller(accounts.charlie);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(400); // 100 * 4
        mp.comprar(pid, 4).unwrap();
        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 3);
    }

    /// Test: Error al marcar como recibido una orden inexistente.
    #[ink::test]
    fn marcar_recibido_orden_inexistente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        assert_eq!(mp.marcar_recibido(999), Err(Error::OrdenInexistente));
    }

    /// Test: Overflow en restauración de stock al aceptar cancelación.
    #[ink::test]
    fn cancelacion_overflow_stock() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                1,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        let mut prod = mp.obtener_producto(pid).unwrap();
        prod.stock = u32::MAX;
        mp.productos.insert(pid, &prod);

        mp.solicitar_cancelacion(oid).unwrap();

        set_next_caller(accounts.alice);
        assert_eq!(mp.aceptar_cancelacion(oid), Err(Error::StockOverflow));
    }

    /// Test: Permisos al marcar como enviado por vendedor distinto al propietario de la orden.
    #[ink::test]
    fn marcar_enviado_otro_vendedor_sin_permiso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.charlie);
        mp.registrar(Rol::Vendedor).unwrap();

        assert_eq!(mp.marcar_enviado(oid), Err(Error::SinPermiso));
    }

    /// Test: Error al solicitar cancelación cuando ya existe una pendiente.
    #[ink::test]
    fn solicitar_cancelacion_ya_pendiente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));

        set_next_caller(accounts.alice);
        assert_eq!(
            mp.solicitar_cancelacion(oid),
            Err(Error::CancelacionYaPendiente)
        );
    }

    /// Test: Error al aceptar cancelación inexistente.
    #[ink::test]
    fn aceptar_cancelacion_inexistente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        assert_eq!(
            mp.aceptar_cancelacion(oid),
            Err(Error::CancelacionInexistente)
        );
    }

    /// Test: Error al aceptar cancelación sin ser el otro participante.
    #[ink::test]
    fn aceptar_cancelacion_sin_permiso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        mp.solicitar_cancelacion(oid).unwrap();

        set_next_caller(accounts.charlie);
        mp.registrar(Rol::Comprador).unwrap();
        assert_eq!(mp.aceptar_cancelacion(oid), Err(Error::SinPermiso));
    }

    /// Test: Error al rechazar cancelación inexistente.
    #[ink::test]
    fn rechazar_cancelacion_inexistente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        assert_eq!(
            mp.rechazar_cancelacion(oid),
            Err(Error::CancelacionInexistente)
        );
    }

    /// Test: Flujo completo de cancelación en estado Enviado.
    #[ink::test]
    fn cancelacion_flujo_completo_estado_enviado() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                5,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(200); // 100 * 2
        let oid = mp.comprar(pid, 2).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));

        set_next_caller(accounts.alice);
        assert_eq!(mp.aceptar_cancelacion(oid), Ok(()));

        assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Cancelada);
        assert_eq!(mp.obtener_producto(pid).unwrap().stock, 5);
    }

    /// Test: Obtener reputación de usuario sin calificaciones.
    #[ink::test]
    fn obtener_reputacion_sin_calificaciones() {
        let accounts = get_accounts();
        let mp = Marketplace::new();

        assert_eq!(mp.obtener_reputacion(accounts.alice), None);
    }

    /// Test: Calificar vendedor exitosamente.
    #[ink::test]
    fn calificar_vendedor_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        mp.marcar_recibido(oid).unwrap();

        assert_eq!(mp.calificar_vendedor(oid, 5), Ok(()));

        let rep = mp.obtener_reputacion(accounts.alice).unwrap();
        assert_eq!(rep.como_vendedor, (5, 1));
    }

    /// Test: Calificar comprador exitosamente.
    #[ink::test]
    fn calificar_comprador_exitoso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        mp.marcar_recibido(oid).unwrap();

        set_next_caller(accounts.alice);
        assert_eq!(mp.calificar_comprador(oid, 4), Ok(()));

        let rep = mp.obtener_reputacion(accounts.bob).unwrap();
        assert_eq!(rep.como_comprador, (4, 1));
    }

    /// Test: Error al calificar vendedor sin ser el comprador.
    #[ink::test]
    fn calificar_vendedor_sin_permiso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        mp.marcar_recibido(oid).unwrap();

        set_next_caller(accounts.charlie);
        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::SinPermiso));
    }

    /// Test: Error al calificar comprador sin ser el vendedor.
    #[ink::test]
    fn calificar_comprador_sin_permiso() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        mp.marcar_recibido(oid).unwrap();

        set_next_caller(accounts.charlie);
        assert_eq!(mp.calificar_comprador(oid, 4), Err(Error::SinPermiso));
    }

    /// Test: Error al calificar orden no recibida.
    #[ink::test]
    fn calificar_orden_no_recibida() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::OrdenNoRecibida));
    }

    /// Test: Error al calificar con puntos inválidos.
    #[ink::test]
    fn calificar_puntos_invalidos() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        mp.marcar_recibido(oid).unwrap();

        assert_eq!(
            mp.calificar_vendedor(oid, 0),
            Err(Error::CalificacionInvalida)
        );
        assert_eq!(
            mp.calificar_vendedor(oid, 6),
            Err(Error::CalificacionInvalida)
        );
    }

    /// Test: Error al calificar dos veces la misma orden.
    #[ink::test]
    fn calificar_dos_veces() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        mp.marcar_recibido(oid).unwrap();

        assert_eq!(mp.calificar_vendedor(oid, 5), Ok(()));
        assert_eq!(mp.calificar_vendedor(oid, 4), Err(Error::YaCalificado));
    }

    /// Test: Calificaciones múltiples acumulan correctamente.
    #[ink::test]
    fn calificaciones_multiples() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid1 = mp
            .publicar(
                "Test1".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();
        let pid2 = mp
            .publicar(
                "Test2".to_string(),
                "Desc".to_string(),
                200,
                10,
                "Cat".to_string(),
            )
            .unwrap();

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
        mp.marcar_recibido(oid1).unwrap();
        mp.marcar_recibido(oid2).unwrap();

        assert_eq!(mp.calificar_vendedor(oid1, 5), Ok(()));
        assert_eq!(mp.calificar_vendedor(oid2, 3), Ok(()));

        let rep = mp.obtener_reputacion(accounts.alice).unwrap();
        assert_eq!(rep.como_vendedor, (8, 2)); // 5 + 3 = 8, count = 2

        let cat = mp
            .obtener_calificacion_categoria("Cat".to_string())
            .unwrap();
        assert_eq!(cat, (8, 2));
    }

    /// Test: Error al calificar orden cancelada.
    #[ink::test]
    fn calificar_orden_cancelada() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        mp.solicitar_cancelacion(oid).unwrap();
        set_next_caller(accounts.alice);
        mp.aceptar_cancelacion(oid).unwrap();

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::OrdenNoRecibida));
    }

    /// Test: Calificar orden inexistente.
    #[ink::test]
    fn calificar_vendedor_orden_inexistente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(999, 5), Err(Error::OrdenInexistente));
    }

    /// Test: Calificar comprador orden inexistente.
    #[ink::test]
    fn calificar_comprador_orden_inexistente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        assert_eq!(mp.calificar_comprador(999, 4), Err(Error::OrdenInexistente));
    }

    /// Test: Ambas partes califican exitosamente.
    #[ink::test]
    fn calificacion_bidireccional_completa() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        mp.marcar_recibido(oid).unwrap();

        assert_eq!(mp.calificar_vendedor(oid, 5), Ok(()));

        set_next_caller(accounts.alice);
        assert_eq!(mp.calificar_comprador(oid, 4), Ok(()));

        let rep_vendedor = mp.obtener_reputacion(accounts.alice).unwrap();
        assert_eq!(rep_vendedor.como_vendedor, (5, 1));

        let rep_comprador = mp.obtener_reputacion(accounts.bob).unwrap();
        assert_eq!(rep_comprador.como_comprador, (4, 1));
    }

    /// Test: Error al calificar en estado Pendiente.
    #[ink::test]
    fn calificar_orden_pendiente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::OrdenNoRecibida));
    }

    /// Test: Error al calificar en estado Enviado.
    #[ink::test]
    fn calificar_orden_enviado() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::OrdenNoRecibida));
    }

    /// Test: Overflow en reputación (simulado).
    #[ink::test]
    fn overflow_reputacion() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        mp.marcar_recibido(oid).unwrap();

        let mut rep = mp
            .reputaciones
            .get(accounts.alice)
            .unwrap_or(ReputacionUsuario {
                como_comprador: (0, 0),
                como_vendedor: (u32::MAX - 2, 1),
            });
        rep.como_vendedor = (u32::MAX - 2, 1);
        mp.reputaciones.insert(accounts.alice, &rep);

        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::IdOverflow));
    }

    /// Test: Overflow en cantidad de calificaciones.
    #[ink::test]
    fn overflow_cantidad_calificaciones() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        mp.marcar_recibido(oid).unwrap();

        let mut rep = mp
            .reputaciones
            .get(accounts.alice)
            .unwrap_or(ReputacionUsuario {
                como_comprador: (0, 0),
                como_vendedor: (10, u32::MAX),
            });
        rep.como_vendedor = (10, u32::MAX);
        mp.reputaciones.insert(accounts.alice, &rep);

        assert_eq!(mp.calificar_vendedor(oid, 5), Err(Error::IdOverflow));
    }

    // ===== TESTS DE SISTEMA DE PAGOS =====

    /// Test: Pago insuficiente al comprar.
    #[ink::test]
    fn pago_insuficiente() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(50); // Debería ser 100
        let resultado = mp.comprar(pid, 1);
        assert_eq!(resultado, Err(Error::PagoInsuficiente));
    }

    /// Test: Pago excesivo al comprar.
    #[ink::test]
    fn pago_excesivo() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(150); // Debería ser 100
        let resultado = mp.comprar(pid, 1);
        assert_eq!(resultado, Err(Error::PagoExcesivo));
    }

    /// Test: Fondos retenidos en escrow después de compra.
    #[ink::test]
    fn fondos_retenidos_despues_de_compra() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(300); // 100 * 3
        let oid = mp.comprar(pid, 3).unwrap();

        // Verificar fondos retenidos
        assert_eq!(mp.obtener_fondos_retenidos(oid), 300);
    }

    /// Test: Fondos liberados al marcar recibido.
    #[ink::test]
    fn fondos_liberados_al_recibir() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(100);
        let oid = mp.comprar(pid, 1).unwrap();

        // Verificar fondos retenidos antes
        assert_eq!(mp.obtener_fondos_retenidos(oid), 100);

        set_next_caller(accounts.alice);
        mp.marcar_enviado(oid).unwrap();

        set_next_caller(accounts.bob);
        // Nota: En unit tests, transfer() no funcionará realmente
        // pero verificamos que se llama correctamente (no panic)
        let _ = mp.marcar_recibido(oid);

        // Los fondos deberían haberse eliminado del mapping (retorna 0)
        assert_eq!(mp.obtener_fondos_retenidos(oid), 0);
    }

    /// Test: Monto total se guarda en la orden.
    #[ink::test]
    fn monto_total_en_orden() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                50,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(250); // 50 * 5
        let oid = mp.comprar(pid, 5).unwrap();

        let orden = mp.obtener_orden(oid).unwrap();
        assert_eq!(orden.monto_total, 250);
    }

    /// Test: Fondos devueltos al cancelar orden.
    #[ink::test]
    fn fondos_devueltos_al_cancelar() {
        let accounts = get_accounts();
        let mut mp = Marketplace::new();

        set_next_caller(accounts.alice);
        mp.registrar(Rol::Vendedor).unwrap();
        let pid = mp
            .publicar(
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10,
                "Cat".to_string(),
            )
            .unwrap();

        set_next_caller(accounts.bob);
        mp.registrar(Rol::Comprador).unwrap();
        set_value(200); // 100 * 2
        let oid = mp.comprar(pid, 2).unwrap();

        // Verificar fondos retenidos
        assert_eq!(mp.obtener_fondos_retenidos(oid), 200);

        // Solicitar y aceptar cancelación
        mp.solicitar_cancelacion(oid).unwrap();
        set_next_caller(accounts.alice);
        // En unit tests, transfer no funcionará realmente
        let _ = mp.aceptar_cancelacion(oid);

        // Los fondos deberían haberse eliminado (retorna 0)
        assert_eq!(mp.obtener_fondos_retenidos(oid), 0);
    }
}
