#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod market {

    use ink::storage::Mapping;
    use ink::prelude::string::String;

    /// Posibles errores que pueden surgir durante la ejecución del contrato.
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// El usuario no tiene rol de vendedor.
        NoEsVendedor,
        /// El usuario no tiene rol de comprador.
        NoEsComprador,
        /// El usuario no está registrado en el sistema.
        NoEstaRegistrado,
        /// El usuario no tiene productos registrados.
        NoTieneProductos,
        /// No hay suficiente stock disponible para la compra.
        StockInsuficiente,
        /// No se encontró el producto con el ID especificado.
        ProductoNoEncontrado,
    }

    /// Enum que representa los posibles roles de un usuario en el marketplace.
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum Rol {
        /// Usuario con rol de comprador.
        #[default]
        Comprador,
        /// Usuario con rol de vendedor.
        Vendedor,
    }

    /// Enum que representa los posibles estados de una orden de compra.
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum EstadoOrden {
        /// La orden ha sido creada pero aún no enviada.
        Pendiente,
        /// El vendedor ha marcado la orden como enviada.
        Enviado,
        /// El comprador ha marcado la orden como recibida.
        Recibido,
        /// La orden fue cancelada.
        Cancelada,
    }

    /// Estructura que representa a un usuario registrado en el sistema.
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Usuario {
        /// Nombre de usuario.
        nombre: String,
        /// Rol del usuario (Comprador o Vendedor).
        rol: Rol,
        /// Reputación acumulada del usuario.
        reputacion: u8,
    }

    /// Estructura que representa un producto publicado por un vendedor.
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Producto {
        /// Identificador único del producto.
        id: u32,
        /// Nombre del producto.
        nombre: String,
        /// Descripción del producto.
        descripcion: String,
        /// Precio del producto en unidades de balance.
        precio: Balance,
        /// Cantidad disponible en stock.
        cantidad: u32,
        /// Categoría del producto.
        categoria: String,
        /// Dirección del vendedor que publicó el producto.
        vendedor: AccountId,
    }

    /// Representa una orden de compra entre comprador y vendedor.
    #[derive(Debug, PartialEq, Eq, Clone)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Orden {
        /// Dirección del comprador.
        comprador: AccountId,
        /// Dirección del vendedor.
        vendedor: AccountId,
        /// ID del producto comprado.
        producto_id: u32,
        /// Cantidad de productos comprados.
        cantidad: u32,
        /// Estado actual de la orden.
        estado: EstadoOrden,
    }

    /// Estructura principal del contrato Marketplace.
    #[ink(storage)]
    pub struct Market {
        /// Dirección del creador del contrato (owner).
        owner: AccountId,
        /// Mapeo de usuarios registrados por su AccountId.
        usuarios: Mapping<AccountId, Usuario>,
        /// Mapeo de productos publicados por ID.
        productos: Mapping<u32, Producto>,
        /// Mapeo de órdenes de compra por ID.
        ordenes: Mapping<u32, Orden>,
        /// Contador para asignar IDs únicos a productos.
        siguiente_id_producto: u32,
        /// Contador para asignar IDs únicos a órdenes.
        siguiente_id_orden: u32,
    }

    impl Market {
        /// Crea una nueva instancia del contrato Marketplace.
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

        /// Devuelve la dirección del creador del contrato.
        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.owner
        }

        /// Registra un nuevo usuario en el sistema con un nombre y un rol inicial.
        ///
        /// # Parámetros
        /// - `nombre`: Nombre del usuario.
        /// - `rol`: Rol asignado al usuario (Comprador o Vendedor).
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

        /// Permite al usuario cambiar su rol actual.
        ///
        /// # Parámetros
        /// - `nuevo_rol`: Nuevo rol que se quiere asignar.
        ///
        /// # Errores
        /// - Retorna `Error::NoEstaRegistrado` si el usuario no está registrado.
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

        /// Verifica si un usuario está registrado en el sistema.
        ///
        /// # Parámetros
        /// - `cuenta`: Dirección del usuario.
        ///
        /// # Retorna
        /// - `Ok(())` si está registrado.
        /// - `Error::NoEstaRegistrado` si no lo está.
        fn esta_registrado(&self, cuenta: AccountId) -> Result<(), Error> {
            if self.usuarios.contains(cuenta) {
                Ok(())
            } else {
                Err(Error::NoEstaRegistrado)
            }
        }

        /// Verifica si un usuario tiene rol de vendedor.
        ///
        /// # Parámetros
        /// - `usuario`: Referencia al usuario a validar.
        ///
        /// # Errores
        /// - Retorna `Error::NoEsVendedor` si el rol no es vendedor.
        fn es_vendedor(&self, usuario: &Usuario) -> Result<(), Error> {
            if usuario.rol == Rol::Vendedor {
                Ok(())
            } else {
                Err(Error::NoEsVendedor)
            }
        }

        /// Verifica si un usuario tiene rol de comprador.
        ///
        /// # Parámetros
        /// - `usuario`: Referencia al usuario a validar.
        ///
        /// # Errores
        /// - Retorna `Error::NoEsComprador` si el rol no es comprador.
        fn es_comprador(&self, usuario: &Usuario) -> Result<(), Error> {
            if usuario.rol == Rol::Comprador {
                Ok(())
            } else {
                Err(Error::NoEsComprador)
            }
        }
    }
}
