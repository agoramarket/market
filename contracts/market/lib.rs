//! # Ágora Marketplace
//! 
//! Un contrato inteligente descentralizado que permite a los usuarios 
//! registrarse como compradores, vendedores o ambos, publicar productos, realizar compras 
//! y gestionar el estado de las órdenes desde la creación hasta la entrega.
//! 
//! ## Características principales
//! 
//! - **Gestión de roles**: Los usuarios pueden registrarse como compradores, vendedores o ambos
//! - **Publicación de productos**: Los vendedores pueden publicar productos con precio y stock
//! - **Sistema de órdenes**: Gestión completa del ciclo de vida de las órdenes
//! - **Control de estados**: Seguimiento del estado de envío y recepción de productos
//! - **Validaciones de seguridad**: Control de permisos y validación de parámetros
//! 
//! ## Flujo de trabajo típico
//! 
//! 1. **Registro**: Los usuarios se registran con un rol específico
//! 2. **Publicación**: Los vendedores publican productos con precio y stock
//! 3. **Compra**: Los compradores adquieren productos generando órdenes
//! 4. **Envío**: Los vendedores marcan las órdenes como enviadas
//! 5. **Recepción**: Los compradores confirman la recepción completando el ciclo
//! 
//! ## Ejemplo de uso completo
//! 
//! ```rust,no_run
//! use marketplace::{Marketplace, Rol, Estado};
//! 
//! // Crear una nueva instancia del marketplace
//! let mut marketplace = Marketplace::new();
//! 
//! // Simular registro de vendedor (Alice)
//! // marketplace.registrar(Rol::Vendedor).unwrap();
//! 
//! // Publicar un producto
//! // let producto_id = marketplace.publicar("Laptop Gaming".to_string(), 1500, 5).unwrap();
//! 
//! // Simular cambio a comprador (Bob) y realizar una compra
//! // marketplace.registrar(Rol::Comprador).unwrap();
//! // let orden_id = marketplace.comprar(producto_id, 2).unwrap();
//! 
//! // Gestionar el estado de la orden
//! // marketplace.marcar_enviado(orden_id).unwrap();  // Como vendedor
//! // marketplace.marcar_recibido(orden_id).unwrap(); // Como comprador
//! ```
//! 
//! ## Seguridad y validaciones
//! 
//! El contrato implementa múltiples capas de validación:
//! - Control de permisos basado en roles
//! - Validación de parámetros de entrada
//! - Verificación de stock antes de compras
//! - Gestión segura de estados de órdenes
//! - Protección contra overflow de IDs

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod marketplace {
    use ink::prelude::string::String;
    use ink::storage::Mapping;

    /// Representa los diferentes roles que puede tener un usuario en el marketplace.
    /// 
    /// Los roles determinan qué acciones puede realizar cada usuario:
    /// - [`Comprador`](Rol::Comprador): Solo puede comprar productos y marcar órdenes como recibidas
    /// - [`Vendedor`](Rol::Vendedor): Solo puede publicar productos y marcar órdenes como enviadas
    /// - [`Ambos`](Rol::Ambos): Puede realizar todas las acciones tanto de comprador como de vendedor
    /// 
    /// # Ejemplos
    /// 
    /// ```rust
    /// # use marketplace::Rol;
    /// let rol_comprador = Rol::Comprador;
    /// assert!(rol_comprador.es_comprador());
    /// assert!(!rol_comprador.es_vendedor());
    /// 
    /// let rol_ambos = Rol::Ambos;
    /// assert!(rol_ambos.es_comprador());
    /// assert!(rol_ambos.es_vendedor());
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Rol {
        /// Usuario que solo puede comprar productos
        Comprador,
        /// Usuario que solo puede vender productos  
        Vendedor,
        /// Usuario que puede tanto comprar como vender productos
        Ambos,
    }

    impl Rol {
        /// Verifica si el rol tiene permisos de comprador.
        /// 
        /// # Returns
        /// 
        /// `true` si el rol es [`Comprador`](Rol::Comprador) o [`Ambos`](Rol::Ambos), `false` en caso contrario.
        /// 
        /// # Ejemplos
        /// 
        /// ```rust
        /// # use marketplace::Rol;
        /// assert!(Rol::Comprador.es_comprador());
        /// assert!(!Rol::Vendedor.es_comprador());
        /// assert!(Rol::Ambos.es_comprador());
        /// ```
        pub fn es_comprador(&self) -> bool {
            matches!(self, Rol::Comprador | Rol::Ambos)
        }

        /// Verifica si el rol tiene permisos de vendedor.
        /// 
        /// # Returns
        /// 
        /// `true` si el rol es [`Vendedor`](Rol::Vendedor) o [`Ambos`](Rol::Ambos), `false` en caso contrario.
        /// 
        /// # Ejemplos
        /// 
        /// ```rust
        /// # use marketplace::Rol;
        /// assert!(!Rol::Comprador.es_vendedor());
        /// assert!(Rol::Vendedor.es_vendedor());
        /// assert!(Rol::Ambos.es_vendedor());
        /// ```
        pub fn es_vendedor(&self) -> bool {
            matches!(self, Rol::Vendedor | Rol::Ambos)
        }
    }

    /// Representa los diferentes estados por los que puede pasar una orden en el marketplace.
    /// 
    /// El ciclo de vida de una orden sigue esta secuencia:
    /// 1. [`Pendiente`](Estado::Pendiente): La orden se ha creado pero el vendedor aún no ha enviado el producto
    /// 2. [`Enviado`](Estado::Enviado): El vendedor ha marcado la orden como enviada
    /// 3. [`Recibido`](Estado::Recibido): El comprador ha confirmado la recepción del producto
    /// 
    /// # Ejemplos
    /// 
    /// ```rust
    /// # use marketplace::Estado;
    /// let estado = Estado::Pendiente;
    /// // Una vez que el vendedor envía el producto
    /// let estado = Estado::Enviado;
    /// // Cuando el comprador confirma la recepción
    /// let estado = Estado::Recibido;
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Estado {
        /// La orden ha sido creada pero el producto aún no ha sido enviado
        Pendiente,
        /// El vendedor ha marcado la orden como enviada
        Enviado,
        /// El comprador ha confirmado la recepción del producto
        Recibido,
    }



    /// Enumera todos los posibles errores que pueden ocurrir en las operaciones del marketplace.
    /// 
    /// Cada variante representa una condición de error específica que puede surgir
    /// durante la ejecución de las diferentes funciones del contrato.
    /// 
    /// # Variantes
    /// 
    /// - [`YaRegistrado`](Error::YaRegistrado): El usuario ya está registrado en el sistema
    /// - [`SinRegistro`](Error::SinRegistro): El usuario no está registrado en el sistema
    /// - [`SinPermiso`](Error::SinPermiso): El usuario no tiene permisos para realizar la acción
    /// - [`ParamInvalido`](Error::ParamInvalido): Los parámetros proporcionados no son válidos
    /// - [`ProdInexistente`](Error::ProdInexistente): El producto especificado no existe
    /// - [`StockInsuf`](Error::StockInsuf): Stock insuficiente para completar la operación
    /// - [`OrdenInexistente`](Error::OrdenInexistente): La orden especificada no existe
    /// - [`EstadoInvalido`](Error::EstadoInvalido): La orden no está en el estado correcto para la operación
    /// - [`IdOverflow`](Error::IdOverflow): Se ha alcanzado el límite máximo de IDs
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        /// El usuario ya está registrado en el marketplace
        YaRegistrado,
        /// El usuario no está registrado en el marketplace
        SinRegistro,
        /// El usuario no tiene permisos para realizar esta acción
        SinPermiso,
        /// Los parámetros proporcionados no son válidos
        ParamInvalido,
        /// El producto especificado no existe
        ProdInexistente,
        /// Stock insuficiente para completar la compra
        StockInsuf,
        /// La orden especificada no existe
        OrdenInexistente,
        /// La orden no está en el estado correcto para esta operación
        EstadoInvalido,
        /// Se ha alcanzado el límite máximo de IDs (overflow)
        IdOverflow,
    }
    /// Estructura principal del contrato que mantiene el estado del marketplace.
    /// 
    /// El `Marketplace` gestiona todos los aspectos del sistema de comercio descentralizado,
    /// incluyendo usuarios, productos y órdenes. Utiliza mappings de ink!, tuplas y enteros sin signo para un
    /// almacenamiento simple y eficiente en la blockchain.
    /// 
    /// # Campos
    /// 
    /// - `roles`: Mapea direcciones de usuario a sus roles (0=Comprador, 1=Vendedor, 2=Ambos)
    /// - `productos`: Mapea IDs de productos a la tupla (vendedor, nombre, precio, stock)
    /// - `ordenes`: Mapea IDs de orden a la tupla (comprador, vendedor, producto_id, cantidad, estado)
    /// - `next_prod_id`: Próximo ID disponible para productos
    /// - `next_order_id`: Próximo ID disponible para órdenes
    /// 
    /// Decidimos utilizar tuplas para los productos y órdenes porque resultaban más simples de manipular que los structs
    /// en esta versión del framework. Esto permite una implementación más directa y eficiente en gas a costa de verbosidad,
    /// pero fué un sacrificio que estábamos dispuestos a hacer.
    /// 
    /// # Ejemplos
    /// 
    /// ```rust
    /// let marketplace = Marketplace::new();
    /// // El marketplace está listo para registrar usuarios y gestionar productos
    /// ``` 
    #[ink(storage)]
    pub struct Marketplace {
        /// Mapeo de direcciones de usuario a sus roles codificados como u8
        roles: Mapping<AccountId, u8>,
        /// Mapeo de IDs de producto a (vendedor, nombre, precio, stock)
        productos: Mapping<u32, (AccountId, String, Balance, u32)>,
        /// Mapeo de IDs de orden a (comprador, vendedor, producto_id, cantidad, estado)
        ordenes: Mapping<u32, (AccountId, AccountId, u32, u32, u8)>,
        /// Próximo ID disponible para asignar a un producto
        next_prod_id: u32,
        /// Próximo ID disponible para asignar a una orden
        next_order_id: u32,
    }

    impl Marketplace {
        /// Crea una nueva instancia del marketplace.
        /// 
        /// Inicializa todos los mappings vacíos y establece los contadores de ID
        /// comenzando desde 1 para productos y órdenes.
        /// 
        /// # Returns
        /// 
        /// Una nueva instancia de [`Marketplace`] lista para usar.
        /// 
        /// # Ejemplos
        /// 
        /// ```rust
        /// # use marketplace::Marketplace;
        /// let marketplace = Marketplace::new();
        /// // El marketplace está inicializado y listo para operaciones
        /// ```
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

        /// Función auxiliar para validar condiciones y retornar errores apropiados.
        /// 
        /// Esta función interna simplifica la validación de condiciones a lo largo
        /// del código, proporcionando una forma consistente de manejar errores.
        /// 
        /// # Arguments
        /// 
        /// * `cond` - La condición a evaluar
        /// * `err` - El error a retornar si la condición es falsa
        /// 
        /// # Returns
        /// 
        /// `Ok(())` si la condición es verdadera, `Err(err)` si es falsa.
        /// 
        /// # Ejemplos
        /// 
        /// ```rust
        /// self.ensure(precio > 0, Error::ParamInvalido)?;
        /// ```
        fn ensure(&self, cond: bool, err: Error) -> Result<(), Error> {
            if cond {
                Ok(())
            } else {
                Err(err)
            }
        }

        /// Obtiene el rol de un usuario basado en su AccountId.
        /// 
        /// Convierte la representación interna u8 del rol a la enumeración `Rol`.
        /// 
        /// # Arguments
        /// 
        /// * `who` - La dirección del usuario cuyo rol se quiere obtener
        /// 
        /// # Returns
        /// 
        /// `Ok(Rol)` si el usuario está registrado, `Err(Error::SinRegistro)` si no.
        /// 
        /// # Errores
        /// 
        /// - `Error::SinRegistro` - El usuario no está registrado en el sistema
        /// 
        /// # Ejemplos
        /// 
        /// ```rust
        /// let rol = marketplace.obtener_rol(user_account)?;
        /// ```
        fn obtener_rol(&self, who: AccountId) -> Result<Rol, Error> {
            match self.roles.get(who) {
                Some(0) => Ok(Rol::Comprador),
                Some(1) => Ok(Rol::Vendedor),
                Some(2) => Ok(Rol::Ambos),
                _ => Err(Error::SinRegistro),
            }
        }

        /// Verifica que un usuario tenga permisos de comprador.
        /// 
        /// # Arguments
        /// 
        /// * `who` - La dirección del usuario a verificar
        /// 
        /// # Returns
        /// 
        /// `Ok(())` si el usuario tiene permisos de comprador, error apropiado si no.
        /// 
        /// # Errores
        /// 
        /// - `Error::SinRegistro` - El usuario no está registrado
        /// - `Error::SinPermiso` - El usuario no tiene permisos de comprador
        fn assert_comprador(&self, who: AccountId) -> Result<(), Error> {
            self.ensure(self.obtener_rol(who)?.es_comprador(), Error::SinPermiso)
        }

        /// Verifica que un usuario tenga permisos de vendedor.
        /// 
        /// # Arguments
        /// 
        /// * `who` - La dirección del usuario a verificar
        /// 
        /// # Returns
        /// 
        /// `Ok(())` si el usuario tiene permisos de vendedor, error apropiado si no.
        /// 
        /// # Errores
        /// 
        /// - `Error::SinRegistro` - El usuario no está registrado
        /// - `Error::SinPermiso` - El usuario no tiene permisos de vendedor
        fn assert_vendedor(&self, who: AccountId) -> Result<(), Error> {
            self.ensure(self.obtener_rol(who)?.es_vendedor(), Error::SinPermiso)
        }

        /// Registra un nuevo usuario en el marketplace con el rol especificado.
        /// 
        /// Este método permite a los usuarios registrarse en el marketplace con uno de los
        /// tres roles disponibles: [`Comprador`](Rol::Comprador), [`Vendedor`](Rol::Vendedor) o [`Ambos`](Rol::Ambos). 
        /// Un usuario solo puede registrarse una vez.
        /// 
        /// # Arguments
        /// 
        /// * `rol` - El rol que el usuario desea tener en el marketplace
        /// 
        /// # Returns
        /// 
        /// `Ok(())` si el registro es exitoso, `Err(Error)` si hay algún problema.
        /// 
        /// # Errores
        /// 
        /// - [`Error::YaRegistrado`] - El usuario ya está registrado en el marketplace
        /// 
        /// # Ejemplos
        /// 
        /// ```rust,no_run
        /// # use marketplace::{Marketplace, Rol};
        /// # let mut marketplace = Marketplace::new();
        /// // Registrar como vendedor
        /// marketplace.registrar(Rol::Vendedor)?;
        /// 
        /// // Registrar como comprador
        /// marketplace.registrar(Rol::Comprador)?;
        /// 
        /// // Registrar con ambos roles
        /// marketplace.registrar(Rol::Ambos)?;
        /// # Ok::<(), marketplace::Error>(())
        /// ```
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

        /// Publica un nuevo producto en el marketplace.
        /// 
        /// Permite a los vendedores registrados publicar productos en el marketplace.
        /// Cada producto tiene un nombre, precio y cantidad de stock disponible.
        /// 
        /// # Arguments
        /// 
        /// * `nombre` - El nombre del producto (máximo 64 caracteres)
        /// * `precio` - El precio del producto (debe ser mayor que 0)
        /// * `stock` - La cantidad disponible del producto (debe ser mayor que 0)
        /// 
        /// # Returns
        /// 
        /// `Ok(u32)` con el ID del producto creado, `Err(Error)` si hay algún problema.
        /// 
        /// # Errores
        /// 
        /// - [`Error::SinRegistro`] - El usuario no está registrado
        /// - [`Error::SinPermiso`] - El usuario no tiene permisos de vendedor
        /// - [`Error::ParamInvalido`] - Precio ≤ 0, stock ≤ 0, o nombre muy largo
        /// - [`Error::IdOverflow`] - Se alcanzó el límite máximo de IDs de producto
        /// 
        /// # Ejemplos
        /// 
        /// ```rust,no_run
        /// # use marketplace::Marketplace;
        /// # let mut marketplace = Marketplace::new();
        /// let producto_id = marketplace.publicar(
        ///     "Laptop Gaming".to_string(),
        ///     1500, // precio
        ///     10    // stock
        /// )?;
        /// # Ok::<(), marketplace::Error>(())
        /// ```
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

        /// Permite a un comprador adquirir una cantidad específica de un producto.
        /// 
        /// Esta función crea una nueva orden de compra, reduce el stock del producto
        /// y asigna un ID único a la orden. La orden comienza en estado `Pendiente`.
        /// 
        /// # Arguments
        /// 
        /// * `id_prod` - El ID del producto a comprar
        /// * `cant` - La cantidad del producto a comprar
        /// 
        /// # Returns
        /// 
        /// `Ok(u32)` con el ID de la orden creada, `Err(Error)` si hay algún problema.
        /// 
        /// # Errores
        /// 
        /// - `Error::SinRegistro` - El usuario no está registrado
        /// - `Error::SinPermiso` - El usuario no tiene permisos de comprador
        /// - `Error::ParamInvalido` - La cantidad es 0
        /// - `Error::ProdInexistente` - El producto no existe
        /// - `Error::StockInsuf` - No hay suficiente stock disponible
        /// - `Error::IdOverflow` - Se alcanzó el límite máximo de IDs de orden
        /// 
        /// # Ejemplos
        /// 
        /// ```rust
        /// let orden_id = marketplace.comprar(
        ///     1, // ID del producto
        ///     3  // cantidad a comprar
        /// )?;
        /// ```
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

        /// Marca una orden como enviada por el vendedor.
        /// 
        /// Solo el vendedor de la orden puede marcar una orden como enviada,
        /// y solo si la orden está en estado `Pendiente`. Después de esto,
        /// la orden pasa al estado `Enviado`.
        /// 
        /// # Arguments
        /// 
        /// * `oid` - El ID de la orden a marcar como enviada
        /// 
        /// # Returns
        /// 
        /// `Ok(())` si la operación es exitosa, `Err(Error)` si hay algún problema.
        /// 
        /// # Errores
        /// 
        /// - `Error::OrdenInexistente` - La orden no existe
        /// - `Error::SinPermiso` - Solo el vendedor puede marcar como enviado
        /// - `Error::EstadoInvalido` - La orden no está en estado `Pendiente`
        /// 
        /// # Ejemplos
        /// 
        /// ```rust
        /// // El vendedor marca la orden como enviada
        /// marketplace.marcar_enviado(orden_id)?;
        /// ```
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

        /// Marca una orden como recibida por el comprador.
        /// 
        /// Solo el comprador de la orden puede marcar una orden como recibida,
        /// y solo si la orden está en estado `Enviado`. Después de esto,
        /// la orden pasa al estado `Recibido`, completando el ciclo de la transacción.
        /// 
        /// # Arguments
        /// 
        /// * `oid` - El ID de la orden a marcar como recibida
        /// 
        /// # Returns
        /// 
        /// `Ok(())` si la operación es exitosa, `Err(Error)` si hay algún problema.
        /// 
        /// # Errores
        /// 
        /// - `Error::OrdenInexistente` - La orden no existe
        /// - `Error::SinPermiso` - Solo el comprador puede marcar como recibido
        /// - `Error::EstadoInvalido` - La orden no está en estado `Enviado`
        /// 
        /// # Ejemplos
        /// 
        /// ```rust
        /// // El comprador confirma la recepción del producto
        /// marketplace.marcar_recibido(orden_id)?;
        /// ```
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

        /// Obtiene la información de un producto por su ID.
        /// 
        /// Esta función de consulta permite obtener todos los detalles de un producto
        /// publicado en el marketplace, incluyendo el vendedor, nombre, precio y stock actual.
        /// 
        /// # Arguments
        /// 
        /// * `id` - El ID del producto a consultar
        /// 
        /// # Returns
        /// 
        /// `Some((AccountId, String, Balance, u32))` con los datos del producto si existe,
        /// `None` si el producto no existe.
        /// 
        /// La tupla retornada contiene:
        /// - `AccountId`: La dirección del vendedor
        /// - `String`: El nombre del producto
        /// - `Balance`: El precio del producto
        /// - `u32`: El stock disponible
        /// 
        /// # Ejemplos
        /// 
        /// ```rust,no_run
        /// # use marketplace::Marketplace;
        /// # let marketplace = Marketplace::new();
        /// if let Some((vendedor, nombre, precio, stock)) = marketplace.get_producto(1) {
        ///     println!("Producto: {} - Precio: {} - Stock: {}", nombre, precio, stock);
        /// }
        /// ```
        #[ink(message)]
        pub fn get_producto(&self, id: u32) -> Option<(AccountId, String, Balance, u32)> {
            self.productos.get(id)
        }

        /// Obtiene la información de una orden por su ID.
        /// 
        /// Esta función de consulta permite obtener todos los detalles de una orden,
        /// incluyendo las partes involucradas, el producto comprado, la cantidad y el estado actual.
        /// 
        /// # Arguments
        /// 
        /// * `id` - El ID de la orden a consultar
        /// 
        /// # Returns
        /// 
        /// `Some((AccountId, AccountId, u32, u32, Estado))` con los datos de la orden si existe,
        /// `None` si la orden no existe.
        /// 
        /// La tupla retornada contiene:
        /// - `AccountId`: La dirección del comprador
        /// - `AccountId`: La dirección del vendedor  
        /// - `u32`: El ID del producto comprado
        /// - `u32`: La cantidad comprada
        /// - [`Estado`]: El estado actual de la orden
        /// 
        /// # Ejemplos
        /// 
        /// ```rust,no_run
        /// # use marketplace::{Marketplace, Estado};
        /// # let marketplace = Marketplace::new();
        /// if let Some((comprador, vendedor, producto_id, cantidad, estado)) = marketplace.get_orden(1) {
        ///     match estado {
        ///         Estado::Pendiente => println!("Orden pendiente de envío"),
        ///         Estado::Enviado => println!("Orden enviada"),
        ///         Estado::Recibido => println!("Orden completada"),
        ///     }
        /// }
        /// ```
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

        /// Obtiene el rol de un usuario registrado en el marketplace.
        /// 
        /// Esta función de consulta permite verificar el rol asignado a cualquier usuario
        /// registrado en el marketplace.
        /// 
        /// # Arguments
        /// 
        /// * `user` - La dirección del usuario cuyo rol se quiere consultar
        /// 
        /// # Returns
        /// 
        /// `Some(Rol)` con el rol del usuario si está registrado,
        /// `None` si el usuario no está registrado.
        /// 
        /// # Ejemplos
        /// 
        /// ```rust,no_run
        /// # use marketplace::{Marketplace, Rol};
        /// # use ink::primitives::AccountId;
        /// # let marketplace = Marketplace::new();
        /// # let user_account = AccountId::from([0x01; 32]);
        /// match marketplace.get_rol(user_account) {
        ///     Some(Rol::Comprador) => println!("Usuario es comprador"),
        ///     Some(Rol::Vendedor) => println!("Usuario es vendedor"),
        ///     Some(Rol::Ambos) => println!("Usuario puede comprar y vender"),
        ///     None => println!("Usuario no registrado"),
        /// }
        /// ```
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

    /// Tests exhaustivos para el contrato del marketplace.
    /// 
    /// Esta sección contiene una suite completa de tests que validan todas las funcionalidades
    /// del marketplace, incluyendo casos de éxito, manejo de errores y validaciones de permisos.
    /// 
    /// Los tests cubren:
    /// - Flujo completo de uso del marketplace
    /// - Validaciones y manejo de errores
    /// - Sistema de roles y permisos
    /// - Estados de órdenes y transiciones
    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        /// Obtiene las cuentas de prueba predeterminadas de ink!
        /// 
        /// # Returns
        /// 
        /// Estructura con cuentas de prueba (alice, bob, charlie, etc.)
        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        /// Establece el caller actual para los tests.
        /// 
        /// # Arguments
        /// 
        /// * `caller` - La cuenta que se establecerá como caller
        fn set_caller(caller: AccountId) {
            test::set_caller::<ink::env::DefaultEnvironment>(caller);
        }

        /// Test del flujo completo del marketplace desde registro hasta entrega.
        /// 
        /// Este test valida el ciclo de vida completo de una transacción en el marketplace:
        /// 1. Registro de vendedor y comprador
        /// 2. Publicación de producto por el vendedor
        /// 3. Compra del producto por el comprador
        /// 4. Marcado como enviado por el vendedor
        /// 5. Marcado como recibido por el comprador
        /// 
        /// También incluye validaciones de errores comunes como intentos de doble registro
        /// y operaciones sin permisos adecuados.
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

        /// Test exhaustivo de validaciones y manejo de errores.
        /// 
        /// Este test verifica que el marketplace maneja correctamente todos los casos de error:
        /// - Operaciones sin registro previo
        /// - Parámetros inválidos en publicación y compra
        /// - Problemas de stock insuficiente
        /// - Errores en gestión de órdenes
        /// - Validaciones de estado en las transiciones
        /// 
        /// Garantiza que el contrato rechace correctamente operaciones inválidas
        /// y retorne los errores apropiados en cada caso.
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

        /// Test del sistema de roles y permisos del marketplace.
        /// 
        /// Este test verifica específicamente el funcionamiento del rol "Ambos",
        /// que permite a un usuario actuar tanto como comprador como vendedor.
        /// 
        /// Valida que:
        /// - Un usuario con rol "Ambos" puede publicar productos (como vendedor)
        /// - El mismo usuario puede comprar productos (como comprador) 
        /// - Puede gestionar el estado de las órdenes desde ambas perspectivas
        /// - Los métodos de consulta funcionan correctamente con datos inexistentes
        /// 
        /// Este test es crucial para verificar la flexibilidad del sistema de roles.
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