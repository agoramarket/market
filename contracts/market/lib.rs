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
}