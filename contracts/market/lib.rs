#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod market {

    use ink::storage::Mapping;
    use ink::prelude::string::String;

    /// Posibles errores del contrato
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        NoEsVendedor,
        NoEsComprador,
        NoEstaRegistrado,
        NoTieneProductos,
        StockInsuficiente,
        ProductoNoEncontrado,
    }

    /// Representa los posibles roles de un usuario
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Rol {
        #[default]
        Comprador,
        Vendedor,
        // Podrías agregar `Ambos` si decides representarlo como enum compuesto.
    }

    /// Estado de una orden de compra
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum EstadoOrden {
        Pendiente,
        Enviado,
        Recibido,
        Cancelada,
    }

    /// Representa a un usuario registrado
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Usuario {
        nombre: String,
        rol: Rol,
        reputacion: u8,
    }

    /// Estructura de un producto publicado
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Producto {
        id: u32,
        nombre: String,
        descripcion: String,
        precio: Balance,
        cantidad: u32,
        categoria: String,
        vendedor: AccountId,
    }

    /// Representa una orden de compra entre comprador y vendedor
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Orden {
        comprador: AccountId,
        vendedor: AccountId,
        producto_id: u32,
        cantidad: u32,
        estado: EstadoOrden,
    }

    /// Almacenamiento del contrato Marketplace
    #[ink(storage)]
    pub struct Market {
        owner: AccountId,
        usuarios: Mapping<AccountId, Usuario>,
        productos: Mapping<u32, Producto>,
        ordenes: Mapping<u32, Orden>,
        siguiente_id_producto: u32,
        siguiente_id_orden: u32,
    }

    impl Market {
        /// Crea una nueva instancia del contrato Marketplace
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                owner: Self::env().caller(),
                usuarios: Mapping::new(),
                productos: Mapping::new(),
                ordenes: Mapping::new(),
                siguiente_id_producto: 1,
                siguiente_id_orden: 1,
            }
        }

        /// Retorna la cuenta del owner (creador del contrato)
        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.owner
        }

        /// Registra un nuevo usuario con un nombre y rol inicial
        #[ink(message)]
        pub fn registrar_usuario(&mut self, nombre: String, rol: Rol) {
            let caller = self.env().caller();
            let usuario = Usuario {
                nombre,
                rol,
                reputacion: 0,
            };
            self.usuarios.insert(caller, &usuario);
        }

        /// Permite cambiar el rol del usuario que llama la función
        #[ink(message)]
        pub fn cambiar_rol(&mut self, nuevo_rol: Rol) -> Result<(), Error> {
            let caller = self.env().caller();
            self.esta_registrado(caller)?;
            let mut usuario = self.usuarios.get(caller).unwrap();
            usuario.rol = nuevo_rol;
            self.usuarios.insert(caller, &usuario);
            Ok(())
        }

        // --- Funciones auxiliares de validación ---

        /// Verifica si un usuario está registrado
        fn esta_registrado(&self, cuenta: AccountId) -> Result<(), Error> {
            if self.usuarios.contains(cuenta) {
                Ok(())
            } else {
                Err(Error::NoEstaRegistrado)
            }
        }

        /// Verifica si un usuario es vendedor
        fn es_vendedor(&self, usuario: &Usuario) -> Result<(), Error> {
            if usuario.rol == Rol::Vendedor {
                Ok(())
            } else {
                Err(Error::NoEsVendedor)
            }
        }

        /// Verifica si un usuario es comprador
        fn es_comprador(&self, usuario: &Usuario) -> Result<(), Error> {
            if usuario.rol == Rol::Comprador {
                Ok(())
            } else {
                Err(Error::NoEsComprador)
            }
        }
    }
}
