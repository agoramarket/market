#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod marketplace {
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use scale::{Decode, Encode};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum Estado {
        Pendiente,
        Enviado,
        Recibido,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Producto {
        pub vendedor: AccountId,
        pub nombre: String,
        pub precio: Balance,
        pub stock: u32,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Orden {
        pub comprador: AccountId,
        pub vendedor: AccountId,
        pub id_prod: u32,
        pub cantidad: u32,
        pub estado: Estado,
    }

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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
        roles: Mapping<AccountId, Rol>,
        productos: Mapping<u32, Producto>,
        ordenes: Mapping<u32, Orden>,
        next_prod_id: u32,
        next_order_id: u32,
    }

    impl Default for Marketplace {
        fn default() -> Self {
            Self::new()
        }
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

        #[ink(message)]
        pub fn registrar(&mut self, rol: Rol) -> Result<(), Error> {
            let caller = self.env().caller();
            self._registrar(caller, rol)
        }

        #[ink(message)]
        pub fn obtener_rol(&self, usuario: AccountId) -> Option<Rol> {
            self.roles.get(usuario)
        }

        #[ink(message)]
        pub fn publicar(
            &mut self,
            nombre: String,
            precio: Balance,
            stock: u32,
        ) -> Result<u32, Error> {
            let vendedor = self.env().caller();
            self._publicar(vendedor, nombre, precio, stock)
        }

        #[ink(message)]
        pub fn obtener_producto(&self, id: u32) -> Option<Producto> {
            self.productos.get(id)
        }

        #[ink(message)]
        pub fn comprar(&mut self, id_prod: u32, cant: u32) -> Result<u32, Error> {
            let comprador = self.env().caller();
            self._comprar(comprador, id_prod, cant)
        }

        #[ink(message)]
        pub fn marcar_enviado(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            self._marcar_enviado(caller, oid)
        }

        #[ink(message)]
        pub fn marcar_recibido(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            self._marcar_recibido(caller, oid)
        }

        #[ink(message)]
        pub fn obtener_orden(&self, id: u32) -> Option<Orden> {
            self.ordenes.get(id)
        }

        fn _registrar(&mut self, caller: AccountId, rol: Rol) -> Result<(), Error> {
            self.ensure(!self.roles.contains(caller), Error::YaRegistrado)?;
            self.roles.insert(caller, &rol);
            Ok(())
        }

        fn _publicar(
            &mut self,
            vendedor: AccountId,
            nombre: String,
            precio: Balance,
            stock: u32,
        ) -> Result<u32, Error> {
            let rol_vendedor = self.rol_de(vendedor)?;
            self.ensure(rol_vendedor.es_vendedor(), Error::SinPermiso)?;
            self.ensure(
                precio > 0 && stock > 0 && nombre.len() <= 64,
                Error::ParamInvalido,
            )?;

            let pid = self.next_prod_id;
            self.next_prod_id = self.next_prod_id.checked_add(1).ok_or(Error::IdOverflow)?;

            let producto = Producto {
                vendedor,
                nombre,
                precio,
                stock,
            };

            self.productos.insert(pid, &producto);
            Ok(pid)
        }

        fn _comprar(&mut self, comprador: AccountId, id_prod: u32, cant: u32) -> Result<u32, Error> {
            let rol_comprador = self.rol_de(comprador)?;
            self.ensure(rol_comprador.es_comprador(), Error::SinPermiso)?;
            self.ensure(cant > 0, Error::ParamInvalido)?;

            let mut producto = self.productos.get(id_prod).ok_or(Error::ProdInexistente)?;
            self.ensure(producto.stock >= cant, Error::StockInsuf)?;

            producto.stock = producto.stock.checked_sub(cant).ok_or(Error::StockInsuf)?;
            self.productos.insert(id_prod, &producto);

            let oid = self.next_order_id;
            self.next_order_id = self.next_order_id.checked_add(1).ok_or(Error::IdOverflow)?;

            let orden = Orden {
                comprador,
                vendedor: producto.vendedor,
                id_prod,
                cantidad: cant,
                estado: Estado::Pendiente,
            };

            self.ordenes.insert(oid, &orden);

            Ok(oid)
        }

        fn _marcar_enviado(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            let mut orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;

            self.ensure(orden.vendedor == caller, Error::SinPermiso)?;
            self.ensure(orden.estado == Estado::Pendiente, Error::EstadoInvalido)?;

            orden.estado = Estado::Enviado;
            self.ordenes.insert(oid, &orden);
            Ok(())
        }

        fn _marcar_recibido(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            let mut orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;

            self.ensure(orden.comprador == caller, Error::SinPermiso)?;
            self.ensure(orden.estado == Estado::Enviado, Error::EstadoInvalido)?;

            orden.estado = Estado::Recibido;
            self.ordenes.insert(oid, &orden);
            Ok(())
        }

        fn ensure(&self, cond: bool, err: Error) -> Result<(), Error> {
            if cond {
                Ok(())
            } else {
                Err(err)
            }
        }

        fn rol_de(&self, quien: AccountId) -> Result<Rol, Error> {
            self.roles.get(quien).ok_or(Error::SinRegistro)
        }
    }
}