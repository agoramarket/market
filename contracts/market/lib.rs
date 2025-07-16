#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod marketplace {
    use ink::prelude::string::String;
    use ink::storage::Mapping;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Rol {
        Comprador,
        Vendedor,
        Ambos,
    }

    impl Rol {
        pub fn es_comprador(&self) -> bool {
            matches!(self, Rol::Comprador | Rol::Ambos)
        }

        pub fn es_vendedor(&self) -> bool {
            matches!(self, Rol::Vendedor | Rol::Ambos)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Estado {
        Pendiente,
        Enviado,
        Recibido,
    }



    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        YaRegistrado,
        SinRegistro,
        SinPermiso,
        ParamInvalido,
        ProdInexistente,
        StockInsuf,
        OrdenInexistente,
        EstadoInvalido,
        IdOverflow,
    }
    #[ink(storage)]
    pub struct Marketplace {
        roles: Mapping<AccountId, u8>,
        productos: Mapping<u32, (AccountId, String, Balance, u32)>,
        ordenes: Mapping<u32, (AccountId, AccountId, u32, u32, u8)>,
        next_prod_id: u32,
        next_order_id: u32,
    }

    impl Marketplace {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                roles: Mapping::default(),
                productos: Mapping::default(),
                ordenes: Mapping::default(),
                next_prod_id: 1,
                next_order_id: 1,
            }
        }

        fn ensure(&self, cond: bool, err: Error) -> Result<(), Error> {
            if cond {
                Ok(())
            } else {
                Err(err)
            }
        }

        fn obtener_rol(&self, who: AccountId) -> Result<Rol, Error> {
            match self.roles.get(who) {
                Some(0) => Ok(Rol::Comprador),
                Some(1) => Ok(Rol::Vendedor),
                Some(2) => Ok(Rol::Ambos),
                _ => Err(Error::SinRegistro),
            }
        }

        fn assert_comprador(&self, who: AccountId) -> Result<(), Error> {
            self.ensure(self.obtener_rol(who)?.es_comprador(), Error::SinPermiso)
        }

        fn assert_vendedor(&self, who: AccountId) -> Result<(), Error> {
            self.ensure(self.obtener_rol(who)?.es_vendedor(), Error::SinPermiso)
        }

        #[ink(message)]
        pub fn registrar(&mut self, rol: Rol) -> Result<(), Error> {
            let caller = self.env().caller();
            self.ensure(!self.roles.contains(caller), Error::YaRegistrado)?;
            let rol_val = match rol {
                Rol::Comprador => 0,
                Rol::Vendedor => 1,
                Rol::Ambos => 2,
            };
            self.roles.insert(caller, &rol_val);
            Ok(())
        }

        #[ink(message)]
        pub fn publicar(
            &mut self,
            nombre: String,
            precio: Balance,
            stock: u32,
        ) -> Result<u32, Error> {
            let vendedor = self.env().caller();
            self.assert_vendedor(vendedor)?;
            self.ensure(precio > 0 && stock > 0, Error::ParamInvalido)?;
            self.ensure(nombre.len() <= 64, Error::ParamInvalido)?;

            let pid = self.next_prod_id;
            self.next_prod_id = self.next_prod_id.checked_add(1).ok_or(Error::IdOverflow)?;

            self.productos.insert(
                pid,
                &(vendedor, nombre, precio, stock),
            );
            Ok(pid)
        }

        #[ink(message)]
        pub fn comprar(&mut self, id_prod: u32, cant: u32) -> Result<u32, Error> {
            let comprador = self.env().caller();
            self.assert_comprador(comprador)?;
            self.ensure(cant > 0, Error::ParamInvalido)?;

            let mut p = self.productos.get(id_prod).ok_or(Error::ProdInexistente)?;
            self.ensure(p.3 >= cant, Error::StockInsuf)?;
            p.3 = p.3.checked_sub(cant).ok_or(Error::StockInsuf)?;
            self.productos.insert(id_prod, &p);

            let oid = self.next_order_id;
            self.next_order_id = self.next_order_id.checked_add(1).ok_or(Error::IdOverflow)?;

            self.ordenes.insert(
                oid,
                &(comprador, p.0, id_prod, cant, 0),
            );
            Ok(oid)
        }

        #[ink(message)]
        pub fn marcar_enviado(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            let mut o = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;
            self.ensure(o.1 == caller, Error::SinPermiso)?;
            self.ensure(o.4 == 0, Error::EstadoInvalido)?;
            o.4 = 1;
            self.ordenes.insert(oid, &o);
            Ok(())
        }

        #[ink(message)]
        pub fn marcar_recibido(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            let mut o = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;
            self.ensure(o.0 == caller, Error::SinPermiso)?;
            self.ensure(o.4 == 1, Error::EstadoInvalido)?;
            o.4 = 2;
            self.ordenes.insert(oid, &o);
            Ok(())
        }

        #[ink(message)]
        pub fn get_producto(&self, id: u32) -> Option<(AccountId, String, Balance, u32)> {
            self.productos.get(id)
        }

        #[ink(message)]
        pub fn get_orden(&self, id: u32) -> Option<(AccountId, AccountId, u32, u32, Estado)> {
            if let Some((comprador, vendedor, id_prod, cantidad, estado_val)) = self.ordenes.get(id) {
                let estado = match estado_val {
                    0 => Estado::Pendiente,
                    1 => Estado::Enviado,
                    2 => Estado::Recibido,
                    _ => Estado::Pendiente,
                };
                Some((comprador, vendedor, id_prod, cantidad, estado))
            } else {
                None
            }
        }

        #[ink(message)]
        pub fn get_rol(&self, user: AccountId) -> Option<Rol> {
            match self.roles.get(user) {
                Some(0) => Some(Rol::Comprador),
                Some(1) => Some(Rol::Vendedor),
                Some(2) => Some(Rol::Ambos),
                _ => None,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        fn set_caller(caller: AccountId) {
            test::set_caller::<ink::env::DefaultEnvironment>(caller);
        }

        #[ink::test]
        fn test_flujo_completo_marketplace() {
            let accounts = default_accounts();
            let mut marketplace = Marketplace::new();
            
            // Test 1: Registro de vendedor
            set_caller(accounts.alice);
            assert_eq!(marketplace.registrar(Rol::Vendedor), Ok(()));
            assert_eq!(marketplace.get_rol(accounts.alice), Some(Rol::Vendedor));
            
            // Test error: ya registrado
            assert_eq!(marketplace.registrar(Rol::Comprador), Err(Error::YaRegistrado));
            
            // Test 2: Registro de comprador
            set_caller(accounts.bob);
            assert_eq!(marketplace.registrar(Rol::Comprador), Ok(()));
            assert_eq!(marketplace.get_rol(accounts.bob), Some(Rol::Comprador));
            
            // Test 3: Publicar producto por vendedor
            set_caller(accounts.alice);
            let producto_id = marketplace.publicar("Laptop".to_string(), 1000, 5).unwrap();
            assert_eq!(producto_id, 1);
            
            let producto = marketplace.get_producto(1).unwrap();
            assert_eq!(producto.0, accounts.alice);
            assert_eq!(producto.1, "Laptop");
            assert_eq!(producto.2, 1000);
            assert_eq!(producto.3, 5);
            
            // Test error: comprador intenta publicar
            set_caller(accounts.bob);
            assert_eq!(marketplace.publicar("Item".to_string(), 100, 1), Err(Error::SinPermiso));
            
            // Test 4: Comprar producto
            let orden_id = marketplace.comprar(1, 2).unwrap();
            assert_eq!(orden_id, 1);
            
            let orden = marketplace.get_orden(1).unwrap();
            assert_eq!(orden.0, accounts.bob); // comprador
            assert_eq!(orden.1, accounts.alice); // vendedor
            assert_eq!(orden.2, 1); // producto_id
            assert_eq!(orden.3, 2); // cantidad
            assert_eq!(orden.4, Estado::Pendiente);
            
            // Verificar que el stock se redujo
            let producto_actualizado = marketplace.get_producto(1).unwrap();
            assert_eq!(producto_actualizado.3, 3); // stock reducido de 5 a 3
            
            // Test 5: Marcar como enviado (solo vendedor)
            set_caller(accounts.alice);
            assert_eq!(marketplace.marcar_enviado(1), Ok(()));
            
            let orden = marketplace.get_orden(1).unwrap();
            assert_eq!(orden.4, Estado::Enviado);
            
            // Test error: comprador intenta marcar como enviado
            set_caller(accounts.bob);
            assert_eq!(marketplace.marcar_enviado(1), Err(Error::SinPermiso));
            
            // Test 6: Marcar como recibido (solo comprador)
            assert_eq!(marketplace.marcar_recibido(1), Ok(()));
            
            let orden = marketplace.get_orden(1).unwrap();
            assert_eq!(orden.4, Estado::Recibido);
        }

        #[ink::test]
        fn test_errores_y_validaciones() {
            let accounts = default_accounts();
            let mut marketplace = Marketplace::new();
            
            // Usuario sin registro intenta acciones
            set_caller(accounts.alice);
            assert_eq!(marketplace.publicar("Item".to_string(), 100, 1), Err(Error::SinRegistro));
            assert_eq!(marketplace.comprar(1, 1), Err(Error::SinRegistro));
            
            // Registrar vendedor y probar validaciones
            marketplace.registrar(Rol::Vendedor).unwrap();
            
            // Parámetros inválidos en publicar
            assert_eq!(marketplace.publicar("".to_string(), 0, 1), Err(Error::ParamInvalido));
            assert_eq!(marketplace.publicar("Item".to_string(), 100, 0), Err(Error::ParamInvalido));
            assert_eq!(marketplace.publicar("a".repeat(65), 100, 1), Err(Error::ParamInvalido));
            
            // Publicar producto válido
            marketplace.publicar("Producto".to_string(), 100, 2).unwrap();
            
            // Registrar comprador
            set_caller(accounts.bob);
            marketplace.registrar(Rol::Comprador).unwrap();
            
            // Errores en comprar
            assert_eq!(marketplace.comprar(999, 1), Err(Error::ProdInexistente)); // producto inexistente
            assert_eq!(marketplace.comprar(1, 0), Err(Error::ParamInvalido)); // cantidad 0
            assert_eq!(marketplace.comprar(1, 10), Err(Error::StockInsuf)); // stock insuficiente
            
            // Compra válida
            marketplace.comprar(1, 2).unwrap(); // compra todo el stock
            
            // Intentar comprar cuando no hay stock
            assert_eq!(marketplace.comprar(1, 1), Err(Error::StockInsuf));
            
            // Errores en órdenes
            assert_eq!(marketplace.marcar_enviado(999), Err(Error::OrdenInexistente));
            assert_eq!(marketplace.marcar_recibido(999), Err(Error::OrdenInexistente));
            
            // Solo vendedor puede marcar como enviado
            assert_eq!(marketplace.marcar_enviado(1), Err(Error::SinPermiso));
            
            set_caller(accounts.alice);
            marketplace.marcar_enviado(1).unwrap();
            
            // No se puede marcar enviado dos veces
            assert_eq!(marketplace.marcar_enviado(1), Err(Error::EstadoInvalido));
            
            // Solo comprador puede marcar como recibido
            assert_eq!(marketplace.marcar_recibido(1), Err(Error::SinPermiso));
        }

        #[ink::test]
        fn test_roles_y_permisos() {
            let accounts = default_accounts();
            let mut marketplace = Marketplace::new();
            
            // Registrar usuario con rol "Ambos"
            set_caller(accounts.alice);
            marketplace.registrar(Rol::Ambos).unwrap();
            assert_eq!(marketplace.get_rol(accounts.alice), Some(Rol::Ambos));
            
            // Usuario con rol "Ambos" puede publicar
            let producto_id = marketplace.publicar("Producto".to_string(), 500, 3).unwrap();
            
            // Usuario con rol "Ambos" puede comprar
            let orden_id = marketplace.comprar(producto_id, 1).unwrap();
            
            // Puede marcar como enviado (como vendedor)
            marketplace.marcar_enviado(orden_id).unwrap();
            
            // Puede marcar como recibido (como comprador)
            marketplace.marcar_recibido(orden_id).unwrap();
            
            // Test métodos de consulta
            assert!(marketplace.get_producto(999).is_none());
            assert!(marketplace.get_orden(999).is_none());
            assert!(marketplace.get_rol(accounts.bob).is_none());
        }
    }
}