#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Ágora Marketplace - 1° Entrega, 18 de julio de 2025.
///
/// Este contrato permite a los usuarios registrarse como compradores, vendedores o ambos.
/// Los vendedores pueden publicar productos, y los compradores pueden comprar esos productos,
/// creando órdenes que siguen un flujo de estado (Pendiente -> Enviado -> Recibido).
#[ink::contract]
mod marketplace {
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use scale::{Decode, Encode};

    /// Define el rol de un usuario en el marketplace.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Rol {
        /// El usuario solo puede comprar productos.
        Comprador,
        /// El usuario solo puede vender productos.
        Vendedor,
        /// El usuario puede comprar y vender productos.
        Ambos,
    }

    impl Rol {
        /// Verifica si el rol permite comprar.
        pub fn es_comprador(&self) -> bool {
            matches!(self, Rol::Comprador | Rol::Ambos)
        }

        /// Verifica si el rol permite vender.
        pub fn es_vendedor(&self) -> bool {
            matches!(self, Rol::Vendedor | Rol::Ambos)
        }
    }

    /// Define el estado de una orden de compra.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Estado {
        /// La orden ha sido creada pero aún no ha sido enviada por el vendedor.
        Pendiente,
        /// El vendedor ha marcado la orden como enviada.
        Enviado,
        /// El comprador ha marcado la orden como recibida.
        Recibido,
    }

    /// Representa un producto en venta en el marketplace.
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Producto {
        /// La cuenta del vendedor que publicó el producto.
        pub vendedor: AccountId,
        /// El nombre del producto.
        pub nombre: String,
        /// El precio del producto.
        pub precio: Balance,
        /// La cantidad de unidades disponibles del producto.
        pub stock: u32,
    }

    /// Representa una orden de compra de un producto.
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Orden {
        /// La cuenta del comprador que realizó la orden.
        pub comprador: AccountId,
        /// La cuenta del vendedor del producto.
        pub vendedor: AccountId,
        /// El identificador del producto comprado.
        pub id_prod: u32,
        /// La cantidad de unidades compradas.
        pub cantidad: u32,
        /// El estado actual de la orden.
        pub estado: Estado,
    }

    /// Enumera los posibles errores que pueden ocurrir en el contrato.
    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// El usuario ya está registrado.
        YaRegistrado,
        /// El usuario no está registrado.
        SinRegistro,
        /// El usuario no tiene el rol adecuado para realizar la acción.
        SinPermiso,
        /// Uno o más parámetros de la función son inválidos.
        ParamInvalido,
        /// El producto especificado no existe.
        ProdInexistente,
        /// No hay suficiente stock del producto para completar la compra.
        StockInsuf,
        /// La orden especificada no existe.
        OrdenInexistente,
        /// La orden no está en el estado correcto para la operación solicitada.
        EstadoInvalido,
        /// El contador de IDs ha alcanzado su valor máximo y no se pueden crear más elementos.
        IdOverflow,
    }

    /// La estructura de almacenamiento principal del contrato.
    #[ink(storage)]
    pub struct Marketplace {
        /// Asigna un rol a cada cuenta de usuario.
        roles: Mapping<AccountId, Rol>,
        /// Almacena los productos publicados, mapeados por su ID.
        productos: Mapping<u32, Producto>,
        /// Almacena las órdenes de compra, mapeadas por su ID.
        ordenes: Mapping<u32, Orden>,
        /// El ID que se asignará al próximo producto publicado.
        next_prod_id: u32,
        /// El ID que se asignará a la próxima orden creada.
        next_order_id: u32,
    }

    impl Default for Marketplace {
        /// Crea una instancia por defecto del `Marketplace`.
        ///
        /// Llamado por `Marketplace::new()` a través del constructor.
        fn default() -> Self {
            Self::new()
        }
    }

    impl Marketplace {
        /// Constructor para crear una nueva instancia del marketplace.
        ///
        /// Inicializa los mappings de almacenamiento y los contadores de IDs.
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

        /// Registra al llamante con un rol específico en el marketplace.
        ///
        /// # Argumentos
        ///
        /// * `rol` - El `Rol` a asignar al llamante (`Comprador`, `Vendedor`, o `Ambos`).
        ///
        /// # Errores
        ///
        /// Devuelve `Error::YaRegistrado` si el llamante ya tiene un rol asignado.
        #[ink(message)]
        pub fn registrar(&mut self, rol: Rol) -> Result<(), Error> {
            let caller = self.env().caller();
            self._registrar(caller, rol)
        }

        /// Obtiene el rol de un usuario específico.
        ///
        /// # Argumentos
        ///
        /// * `usuario` - La `AccountId` del usuario a consultar.
        ///
        /// # Retorno
        ///
        /// Devuelve `Some(Rol)` si el usuario está registrado, o `None` en caso contrario.
        #[ink(message)]
        pub fn obtener_rol(&self, usuario: AccountId) -> Option<Rol> {
            self.roles.get(usuario)
        }

        /// Publica un nuevo producto en el marketplace.
        ///
        /// El llamante debe estar registrado como `Vendedor` o `Ambos`.
        ///
        /// # Argumentos
        ///
        /// * `nombre` - El nombre del producto (máximo 64 caracteres).
        /// * `precio` - El precio del producto (debe ser mayor que 0).
        /// * `stock` - La cantidad de unidades disponibles (debe ser mayor que 0).
        ///
        /// # Errores
        ///
        /// - `Error::SinPermiso` si el llamante no es un vendedor.
        /// - `Error::ParamInvalido` si el precio, stock o nombre no son válidos.
        /// - `Error::IdOverflow` si se ha alcanzado el número máximo de productos.
        ///
        /// # Retorno
        ///
        /// Devuelve el `id` del nuevo producto publicado.
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

        /// Obtiene la información de un producto por su ID.
        ///
        /// # Argumentos
        ///
        /// * `id` - El ID del producto a consultar.
        ///
        /// # Retorno
        ///
        /// Devuelve `Some(Producto)` si el producto existe, o `None` en caso contrario.
        #[ink(message)]
        pub fn obtener_producto(&self, id: u32) -> Option<Producto> {
            self.productos.get(id)
        }

        /// Permite a un comprador crear una orden para un producto.
        ///
        /// El llamante debe estar registrado como `Comprador` o `Ambos`.
        ///
        /// # Argumentos
        ///
        /// * `id_prod` - El ID del producto a comprar.
        /// * `cant` - La cantidad de unidades a comprar (debe ser mayor que 0).
        ///
        /// # Errores
        ///
        /// - `Error::SinPermiso` si el llamante no es un comprador.
        /// - `Error::ParamInvalido` si la cantidad es 0.
        /// - `Error::ProdInexistente` si el producto no existe.
        /// - `Error::StockInsuf` si no hay suficiente stock para la cantidad solicitada.
        /// - `Error::IdOverflow` si se ha alcanzado el número máximo de órdenes.
        ///
        /// # Retorno
        ///
        /// Devuelve el `id` de la nueva orden creada.
        #[ink(message)]
        pub fn comprar(&mut self, id_prod: u32, cant: u32) -> Result<u32, Error> {
            let comprador = self.env().caller();
            self._comprar(comprador, id_prod, cant)
        }

        /// Marca una orden como enviada.
        ///
        /// Solo el vendedor de la orden puede llamar a esta función.
        /// La orden debe estar en estado `Pendiente`.
        ///
        /// # Argumentos
        ///
        /// * `oid` - El ID de la orden a marcar como enviada.
        ///
        /// # Errores
        ///
        /// - `Error::OrdenInexistente` si la orden no existe.
        /// - `Error::SinPermiso` si el llamante no es el vendedor de la orden.
        /// - `Error::EstadoInvalido` si la orden no está en estado `Pendiente`.
        #[ink(message)]
        pub fn marcar_enviado(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            self._marcar_enviado(caller, oid)
        }

        /// Marca una orden como recibida.
        ///
        /// Solo el comprador de la orden puede llamar a esta función.
        /// La orden debe estar en estado `Enviado`.
        ///
        /// # Argumentos
        ///
        /// * `oid` - El ID de la orden a marcar como recibida.
        ///
        /// # Errores
        ///
        /// - `Error::OrdenInexistente` si la orden no existe.
        /// - `Error::SinPermiso` si el llamante no es el comprador de la orden.
        /// - `Error::EstadoInvalido` si la orden no está en estado `Enviado`.
        #[ink(message)]
        pub fn marcar_recibido(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            self._marcar_recibido(caller, oid)
        }

        /// Obtiene la información de una orden por su ID.
        ///
        /// # Argumentos
        ///
        /// * `id` - El ID de la orden a consultar.
        ///
        /// # Retorno
        ///
        /// Devuelve `Some(Orden)` si la orden existe, o `None` en caso contrario.
        #[ink(message)]
        pub fn obtener_orden(&self, id: u32) -> Option<Orden> {
            self.ordenes.get(id)
        }

        /// Lógica interna para registrar un usuario.
        fn _registrar(&mut self, caller: AccountId, rol: Rol) -> Result<(), Error> {
            // Asegura que el usuario (caller) no esté ya registrado. Si lo está, devuelve `Error::YaRegistrado`.
            self.ensure(!self.roles.contains(caller), Error::YaRegistrado)?;
            // Inserta el nuevo rol para el usuario en el mapping `roles`.
            self.roles.insert(caller, &rol);
            // Devuelve `Ok` para indicar que el registro fue exitoso.
            Ok(())
        }

        /// Lógica interna para publicar un producto.
        fn _publicar(
            &mut self,
            vendedor: AccountId,
            nombre: String,
            precio: Balance,
            stock: u32,
        ) -> Result<u32, Error> {
            // Obtiene el rol del vendedor. Devuelve `Error::SinRegistro` si no está registrado.
            let rol_vendedor = self.rol_de(vendedor)?;
            // Asegura que el usuario tenga permisos de vendedor. Si no, devuelve `Error::SinPermiso`.
            self.ensure(rol_vendedor.es_vendedor(), Error::SinPermiso)?;
            // Valida los parámetros del producto. Si son inválidos, devuelve `Error::ParamInvalido`.
            self.ensure(
                precio > 0 && stock > 0 && nombre.len() <= 64,
                Error::ParamInvalido,
            )?;

            // Obtiene el ID para el nuevo producto.
            let pid = self.next_prod_id;
            // Incrementa el contador para el próximo ID de producto, manejando un posible desbordamiento.
            self.next_prod_id = self.next_prod_id.checked_add(1).ok_or(Error::IdOverflow)?;

            // Crea una nueva instancia de `Producto` con los datos proporcionados.
            let producto = Producto {
                vendedor,
                nombre,
                precio,
                stock,
            };

            // Inserta el nuevo producto en el mapping `productos`.
            self.productos.insert(pid, &producto);
            // Devuelve el ID del producto recién creado.
            Ok(pid)
        }

        /// Lógica interna para comprar un producto.
        fn _comprar(
            &mut self,
            comprador: AccountId,
            id_prod: u32,
            cant: u32,
        ) -> Result<u32, Error> {
            // Obtiene el rol del comprador. Devuelve `Error::SinRegistro` si no está registrado.
            let rol_comprador = self.rol_de(comprador)?;
            // Asegura que el usuario tenga permisos de comprador. Si no, devuelve `Error::SinPermiso`.
            self.ensure(rol_comprador.es_comprador(), Error::SinPermiso)?;
            // Asegura que la cantidad a comprar sea mayor que cero.
            self.ensure(cant > 0, Error::ParamInvalido)?;

            // Obtiene el producto a comprar. Si no existe, devuelve `Error::ProdInexistente`.
            let mut producto = self.productos.get(id_prod).ok_or(Error::ProdInexistente)?;
            // Verifica que haya suficiente stock. Si no, devuelve `Error::StockInsuf`.
            self.ensure(producto.stock >= cant, Error::StockInsuf)?;

            // Reduce el stock del producto y maneja un posible subdesbordamiento.
            producto.stock = producto.stock.checked_sub(cant).ok_or(Error::StockInsuf)?;
            // Actualiza la información del producto en el almacenamiento.
            self.productos.insert(id_prod, &producto);

            // Obtiene el ID para la nueva orden.
            let oid = self.next_order_id;
            // Incrementa el contador para el próximo ID de orden, manejando un posible desbordamiento.
            self.next_order_id = self.next_order_id.checked_add(1).ok_or(Error::IdOverflow)?;

            // Crea una nueva instancia de `Orden`.
            let orden = Orden {
                comprador,
                vendedor: producto.vendedor,
                id_prod,
                cantidad: cant,
                estado: Estado::Pendiente,
            };

            // Inserta la nueva orden en el mapping `ordenes`.
            self.ordenes.insert(oid, &orden);

            // Devuelve el ID de la orden recién creada.
            Ok(oid)
        }

        /// Lógica interna para marcar una orden como enviada.
        fn _marcar_enviado(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            // Obtiene la orden. Si no existe, devuelve `Error::OrdenInexistente`.
            let mut orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;

            // Asegura que quien llama es el vendedor de la orden. Si no, devuelve `Error::SinPermiso`.
            self.ensure(orden.vendedor == caller, Error::SinPermiso)?;
            // Asegura que la orden esté en estado `Pendiente`. Si no, devuelve `Error::EstadoInvalido`.
            self.ensure(orden.estado == Estado::Pendiente, Error::EstadoInvalido)?;

            // Cambia el estado de la orden a `Enviado`.
            orden.estado = Estado::Enviado;
            // Actualiza la orden en el almacenamiento.
            self.ordenes.insert(oid, &orden);
            Ok(())
        }

        /// Lógica interna para marcar una orden como recibida.
        fn _marcar_recibido(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            // Obtiene la orden. Si no existe, devuelve `Error::OrdenInexistente`.
            let mut orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;

            // Asegura que quien llama es el comprador de la orden. Si no, devuelve `Error::SinPermiso`.
            self.ensure(orden.comprador == caller, Error::SinPermiso)?;
            // Asegura que la orden esté en estado `Enviado`. Si no, devuelve `Error::EstadoInvalido`.
            self.ensure(orden.estado == Estado::Enviado, Error::EstadoInvalido)?;

            // Cambia el estado de la orden a `Recibido`.
            orden.estado = Estado::Recibido;
            // Actualiza la orden en el almacenamiento.
            self.ordenes.insert(oid, &orden);
            Ok(())
        }

        /// Función de utilidad para verificar una condición y devolver un error si es falsa.
        fn ensure(&self, cond: bool, err: Error) -> Result<(), Error> {
            // Si la condición es verdadera, devuelve `Ok`.
            if cond {
                Ok(())
            } else {
                // Si la condición es falsa, devuelve el error especificado.
                Err(err)
            }
        }

        /// Función de utilidad para obtener el rol de un usuario o devolver `Error::SinRegistro`.
        fn rol_de(&self, quien: AccountId) -> Result<Rol, Error> {
            // Intenta obtener el rol del usuario. Si no existe, devuelve `Error::SinRegistro`.
            self.roles.get(quien).ok_or(Error::SinRegistro)
        }
    }
}
