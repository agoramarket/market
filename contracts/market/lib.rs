#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Ágora Marketplace
///
/// Contrato market - Lógica de compras y ventas en un marketplace descentralizado.
///
/// Este contrato permite a los usuarios registrarse como compradores, vendedores o ambos.
/// Los vendedores pueden publicar productos, y los compradores pueden comprar esos productos,
/// creando órdenes que siguen un flujo de estado (Pendiente -> Enviado -> Recibido).
#[ink::contract]
mod marketplace {
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
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
        /// La orden ha sido cancelada por acuerdo mutuo.
        Cancelada,
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
        /// Descripción detallada del producto.
        pub descripcion: String,
        /// El precio del producto.
        pub precio: Balance,
        /// La cantidad de unidades disponibles del producto.
        pub stock: u32,
        /// Categoría del producto.
        pub categoria: String,
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

    /// Representa una solicitud de cancelación pendiente para una orden.
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct CancelacionPendiente {
        /// El ID de la orden que se desea cancelar.
        pub oid: u32,
        /// La cuenta del participante que solicita la cancelación.
        pub solicitante: AccountId,
    }

    /// Representa la reputación de un usuario en el marketplace.
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct ReputacionUsuario {
        /// Reputación como comprador: (suma de calificaciones, cantidad de calificaciones).
        /// Para obtener el promedio: suma / cantidad
        /// Ejemplo: (15, 3) = promedio de 5.0 estrellas
        pub como_comprador: (u32, u32),
        /// Reputación como vendedor: (suma de calificaciones, cantidad de calificaciones).
        /// Para obtener el promedio: suma / cantidad
        /// Ejemplo: (12, 4) = promedio de 3.0 estrellas
        pub como_vendedor: (u32, u32),
    }

    /// Representa el estado de calificaciones para una orden.
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct CalificacionOrden {
        /// Indica si el comprador ya ha calificado al vendedor.
        pub comprador_califico: bool,
        /// Indica si el vendedor ya ha calificado al comprador.
        pub vendedor_califico: bool,
    }

    /// Límites de longitud para strings en el contrato.
    const MAX_NOMBRE_LEN: usize = 64;
    const MAX_DESCRIPCION_LEN: usize = 256;
    const MAX_CATEGORIA_LEN: usize = 32;

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
        /// Ya existe una solicitud de cancelación pendiente para esta orden.
        CancelacionYaPendiente,
        /// No existe una solicitud de cancelación pendiente para esta orden.
        CancelacionInexistente,
        /// Un usuario intenta comprarse a sí mismo (vendedor intenta comprar su propio producto).
        AutoCompraProhibida,
        /// La orden ha sido cancelada y no puede ser modificada.
        OrdenCancelada,
        /// El solicitante de cancelación no puede aceptar o rechazar su propia solicitud; debe hacerlo el otro participante.
        SolicitanteCancelacion,
        /// El stock del producto excedería el valor máximo al intentar restaurarlo (overflow).
        StockOverflow,
        /// Ya se ha calificado en esta orden.
        YaCalificado,
        /// La calificación debe estar entre 1 y 5.
        CalificacionInvalida,
        /// Solo se puede calificar si la orden está en estado Recibido.
        OrdenNoRecibida,
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
        /// Almacena las solicitudes de cancelación pendientes, mapeadas por el ID de orden.
        cancelaciones_pendientes: Mapping<u32, CancelacionPendiente>,
        /// Almacena la reputación de cada usuario.
        reputaciones: Mapping<AccountId, ReputacionUsuario>,
        /// Almacena el estado de calificaciones para cada orden.
        calificaciones: Mapping<u32, CalificacionOrden>,
        /// Suma y cantidad de calificaciones de vendedores por categoría (promedio = suma / cantidad).
        calificaciones_por_categoria: Mapping<String, (u32, u32)>,
        /// El ID que se asignará al próximo producto publicado.
        next_prod_id: u32,
        /// El ID que se asignará a la próxima orden creada.
        next_order_id: u32,
        /// Lista de todos los usuarios registrados (para iterar en reportes)
        usuarios_registrados: Vec<AccountId>,
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
                cancelaciones_pendientes: Mapping::default(),
                reputaciones: Mapping::default(),
                calificaciones: Mapping::default(),
                calificaciones_por_categoria: Mapping::default(),
                next_prod_id: 1,
                next_order_id: 1,
                usuarios_registrados: Vec::new(),
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

        /// Modifica el rol de un usuario ya registrado.
        ///
        /// El usuario debe estar previamente registrado para poder modificar su rol.
        /// Esta función permite que un usuario cambie de `Comprador` a `Vendedor`,
        /// de `Vendedor` a `Comprador`, o que cualquiera de ellos cambie a `Ambos`.
        ///
        /// # Argumentos
        ///
        /// * `nuevo_rol` - El nuevo `Rol` a asignar al llamante.
        ///
        /// # Errores
        ///
        /// Devuelve `Error::SinRegistro` si el llamante no está registrado previamente.
        #[ink(message)]
        pub fn modificar_rol(&mut self, nuevo_rol: Rol) -> Result<(), Error> {
            let caller = self.env().caller();
            self._modificar_rol(caller, nuevo_rol)
        }

        /// Publica un nuevo producto en el marketplace.
        ///
        /// El llamante debe estar registrado como `Vendedor` o `Ambos`.
        ///
        /// # Argumentos
        ///
        /// * `nombre` - El nombre del producto (máximo 64 caracteres).
        /// * `descripcion` - Descripción del producto (máximo 256 caracteres).
        /// * `precio` - El precio del producto (debe ser mayor que 0).
        /// * `stock` - La cantidad de unidades disponibles (debe ser mayor que 0).
        /// * `categoria` - Categoría del producto (máximo 32 caracteres).
        ///
        /// # Errores
        ///
        /// - `Error::SinPermiso` si el llamante no es un vendedor.
        /// - `Error::ParamInvalido` si el precio, stock, nombre, descripción o categoría no son válidos.
        /// - `Error::IdOverflow` si se ha alcanzado el número máximo de productos.
        ///
        /// # Retorno
        ///
        /// Devuelve el `id` del nuevo producto publicado.
        #[ink(message)]
        pub fn publicar(
            &mut self,
            nombre: String,
            descripcion: String,
            precio: Balance,
            stock: u32,
            categoria: String,
        ) -> Result<u32, Error> {
            let vendedor = self.env().caller();
            self._publicar(vendedor, nombre, descripcion, precio, stock, categoria)
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
        /// Solo el comprador o el vendedor de la orden pueden acceder a esta información.
        ///
        /// # Argumentos
        ///
        /// * `id` - El ID de la orden a consultar.
        ///
        /// # Errores
        ///
        /// - `Error::OrdenInexistente` si la orden no existe.
        /// - `Error::SinPermiso` si el llamante no es el comprador ni el vendedor de la orden.
        ///
        /// # Retorno
        ///
        /// Devuelve la `Orden` si existe y el llamante tiene permisos.
        #[ink(message)]
        pub fn obtener_orden(&self, id: u32) -> Result<Orden, Error> {
            let caller = self.env().caller();
            let orden = self.ordenes.get(id).ok_or(Error::OrdenInexistente)?;
            self.ensure(
                orden.comprador == caller || orden.vendedor == caller,
                Error::SinPermiso,
            )?;
            Ok(orden)
        }

        /// Lista todos los productos publicados por un vendedor específico.
        ///
        /// # Argumentos
        ///
        /// * `vendedor` - La `AccountId` del vendedor cuyos productos se desean listar.
        ///
        /// # Retorno
        ///
        /// Devuelve un `Vec<Producto>` con todos los productos del vendedor.
        /// Si el vendedor no tiene productos, devuelve un vector vacío.
        ///
        /// # Nota
        ///
        /// Esta función itera sobre todos los IDs de productos, por lo que su costo
        /// aumenta linealmente con el número total de productos en el marketplace.
        #[ink(message)]
        pub fn listar_productos_de_vendedor(&self, vendedor: AccountId) -> Vec<Producto> {
            self._listar_productos_de_vendedor(vendedor)
        }

        /// Lista todas las órdenes realizadas por el usuario que llama esta función.
        ///
        /// Por motivos de seguridad y privacidad, un comprador solo puede ver sus propias órdenes.
        ///
        /// # Retorno
        ///
        /// Devuelve un `Vec<Orden>` con todas las órdenes del caller.
        /// Si el caller no tiene órdenes, devuelve un vector vacío.
        ///
        /// # Nota
        ///
        /// Esta función itera sobre todos los IDs de órdenes, por lo que su costo
        /// aumenta linealmente con el número total de órdenes en el marketplace.
        #[ink(message)]
        pub fn listar_ordenes_de_comprador(&self, comprador: AccountId) -> Vec<Orden> {
            self._listar_ordenes_de_comprador(comprador)
        }

        /// Solicita la cancelación de una orden.
        ///
        /// El llamante debe ser el comprador o el vendedor de la orden.
        /// La orden debe estar en estado `Pendiente` o `Enviado`.
        ///
        /// - Si la orden está `Pendiente` y el llamante es el comprador, la orden se
        ///   cancela de forma inmediata y se restaura el stock (camino unilateral
        ///   pedido por la consigna).
        /// - En cualquier otro caso (`Enviado` o petición iniciada por el vendedor),
        ///   se registra una solicitud que debe ser aceptada o rechazada por la otra
        ///   parte. Solo puede haber una solicitud pendiente por orden.
        ///
        /// # Argumentos
        ///
        /// * `oid` - El ID de la orden a cancelar.
        ///
        /// # Errores
        ///
        /// - `Error::OrdenInexistente` si la orden no existe.
        /// - `Error::SinPermiso` si el llamante no es el comprador ni el vendedor.
        /// - `Error::EstadoInvalido` si la orden no está en estado `Pendiente` o `Enviado`.
        /// - `Error::CancelacionYaPendiente` si ya existe una solicitud de cancelación.
        #[ink(message)]
        pub fn solicitar_cancelacion(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            self._solicitar_cancelacion(caller, oid)
        }

        /// Acepta una solicitud de cancelación de una orden.
        ///
        /// El llamante debe ser el otro participante (comprador si vendedor solicita, o viceversa).
        /// Al aceptar, la orden pasa a estado `Cancelada` y el stock se restaura.
        ///
        /// # Argumentos
        ///
        /// * `oid` - El ID de la orden cuya cancelación se desea aceptar.
        ///
        /// # Errores
        ///
        /// - `Error::CancelacionInexistente` si no existe solicitud de cancelación.
        /// - `Error::SinPermiso` si el llamante no es el otro participante.
        /// - `Error::OrdenInexistente` si la orden no existe.
        /// - `Error::ProdInexistente` si el producto no existe.
        #[ink(message)]
        pub fn aceptar_cancelacion(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            self._aceptar_cancelacion(caller, oid)
        }

        /// Rechaza una solicitud de cancelación de una orden.
        ///
        /// El llamante debe ser el otro participante (comprador si vendedor solicita, o viceversa).
        /// Solo elimina la solicitud de cancelación, la orden mantiene su estado anterior.
        ///
        /// # Argumentos
        ///
        /// * `oid` - El ID de la orden cuya cancelación se desea rechazar.
        ///
        /// # Errores
        ///
        /// - `Error::CancelacionInexistente` si no existe solicitud de cancelación.
        /// - `Error::SinPermiso` si el llamante no es el otro participante.
        #[ink(message)]
        pub fn rechazar_cancelacion(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            self._rechazar_cancelacion(caller, oid)
        }

        /// Obtiene la reputación de un usuario específico.
        ///
        /// # Argumentos
        ///
        /// * `usuario` - La `AccountId` del usuario cuya reputación se desea consultar.
        ///
        /// # Retorno
        ///
        /// Devuelve `Some(ReputacionUsuario)` si el usuario tiene reputación registrada, o `None` en caso contrario.
        #[ink(message)]
        pub fn obtener_reputacion(&self, usuario: AccountId) -> Option<ReputacionUsuario> {
            self.reputaciones.get(usuario)
        }

        /// Obtiene la suma y cantidad de calificaciones de vendedores para una categoría.
        /// Retorna `Some((suma, cantidad))` o `None` si aún no hay calificaciones registradas.
        #[ink(message)]
        pub fn obtener_calificacion_categoria(&self, categoria: String) -> Option<(u32, u32)> {
            self.calificaciones_por_categoria.get(categoria)
        }

        /// Permite al comprador calificar al vendedor de una orden.
        ///
        /// Solo el comprador de la orden puede calificar al vendedor.
        /// La orden debe estar en estado `Recibido`.
        /// Solo se puede calificar una vez por orden.
        /// La calificación debe estar entre 1 y 5.
        ///
        /// # Argumentos
        ///
        /// * `oid` - El ID de la orden a calificar.
        /// * `puntos` - La calificación (1-5).
        ///
        /// # Errores
        ///
        /// - `Error::OrdenInexistente` si la orden no existe.
        /// - `Error::SinPermiso` si el llamante no es el comprador de la orden.
        /// - `Error::OrdenNoRecibida` si la orden no está en estado Recibido.
        /// - `Error::YaCalificado` si ya se ha calificado en esta orden.
        /// - `Error::CalificacionInvalida` si los puntos no están entre 1 y 5.
        #[ink(message)]
        pub fn calificar_vendedor(&mut self, oid: u32, puntos: u8) -> Result<(), Error> {
            let caller = self.env().caller();
            self._calificar_vendedor(caller, oid, puntos)
        }

        /// Permite al vendedor calificar al comprador de una orden.
        ///
        /// Solo el vendedor de la orden puede calificar al comprador.
        /// La orden debe estar en estado `Recibido`.
        /// Solo se puede calificar una vez por orden.
        /// La calificación debe estar entre 1 y 5.
        ///
        /// # Argumentos
        ///
        /// * `oid` - El ID de la orden a calificar.
        /// * `puntos` - La calificación (1-5).
        ///
        /// # Errores
        ///
        /// - `Error::OrdenInexistente` si la orden no existe.
        /// - `Error::SinPermiso` si el llamante no es el vendedor de la orden.
        /// - `Error::OrdenNoRecibida` si la orden no está en estado Recibido.
        /// - `Error::YaCalificado` si ya se ha calificado en esta orden.
        /// - `Error::CalificacionInvalida` si los puntos no están entre 1 y 5.
        #[ink(message)]
        pub fn calificar_comprador(&mut self, oid: u32, puntos: u8) -> Result<(), Error> {
            let caller = self.env().caller();
            self._calificar_comprador(caller, oid, puntos)
        }

        /// Obtiene el total de productos publicados.
        /// Útil para que ReportesView pueda iterar sobre todos los productos.
        #[ink(message)]
        pub fn get_total_productos(&self) -> u32 {
            self.next_prod_id.saturating_sub(1)
        }

        /// Obtiene el total de órdenes creadas.
        /// Útil para que ReportesView pueda iterar sobre todas las órdenes.
        #[ink(message)]
        pub fn get_total_ordenes(&self) -> u32 {
            self.next_order_id.saturating_sub(1)
        }

        /// Obtiene una orden por su ID sin restricción de permisos.
        /// Esta función es pública para permitir reportes y análisis.
        ///
        /// # Argumentos
        /// * `id` - El ID de la orden a consultar.
        ///
        /// # Retorno
        /// Devuelve `Some(Orden)` si existe, `None` en caso contrario.
        #[ink(message)]
        pub fn obtener_orden_publica(&self, id: u32) -> Option<Orden> {
            self.ordenes.get(id)
        }

        /// Obtiene la lista de todos los usuarios registrados.
        /// Útil para calcular rankings de reputación.
        #[ink(message)]
        pub fn listar_usuarios(&self) -> Vec<AccountId> {
            self.usuarios_registrados.clone()
        }

        /// Obtiene todos los productos (para reportes).
        /// Itera internamente y devuelve la lista completa.
        #[ink(message)]
        pub fn listar_todos_productos(&self) -> Vec<(u32, Producto)> {
            let mut productos = Vec::new();
            for pid in 1..self.next_prod_id {
                if let Some(producto) = self.productos.get(pid) {
                    productos.push((pid, producto));
                }
            }
            productos
        }

        /// Obtiene todas las órdenes (para reportes).
        /// Itera internamente y devuelve la lista completa.
        #[ink(message)]
        pub fn listar_todas_ordenes(&self) -> Vec<(u32, Orden)> {
            let mut ordenes = Vec::new();
            for oid in 1..self.next_order_id {
                if let Some(orden) = self.ordenes.get(oid) {
                    ordenes.push((oid, orden));
                }
            }
            ordenes
        }

        /// Lógica interna para listar productos de un vendedor.
        fn _listar_productos_de_vendedor(&self, vendedor: AccountId) -> Vec<Producto> {
            let mut productos_vendedor = Vec::new();

            for pid in 1..self.next_prod_id {
                if let Some(producto) = self.productos.get(pid) {
                    if producto.vendedor == vendedor {
                        productos_vendedor.push(producto);
                    }
                }
            }

            productos_vendedor
        }

        /// Lógica interna para listar órdenes de un comprador.
        fn _listar_ordenes_de_comprador(&self, comprador: AccountId) -> Vec<Orden> {
            let mut ordenes_comprador = Vec::new();

            for oid in 1..self.next_order_id {
                if let Some(orden) = self.ordenes.get(oid) {
                    if orden.comprador == comprador {
                        ordenes_comprador.push(orden);
                    }
                }
            }

            ordenes_comprador
        }

        /// Lógica interna para registrar un usuario.
        fn _registrar(&mut self, caller: AccountId, rol: Rol) -> Result<(), Error> {
            self.ensure(!self.roles.contains(caller), Error::YaRegistrado)?;
            self.roles.insert(caller, &rol);
            self.usuarios_registrados.push(caller);
            Ok(())
        }

        /// Lógica interna para modificar el rol de un usuario.
        fn _modificar_rol(&mut self, caller: AccountId, nuevo_rol: Rol) -> Result<(), Error> {
            self.ensure(self.roles.contains(caller), Error::SinRegistro)?;
            self.roles.insert(caller, &nuevo_rol);
            Ok(())
        }

        /// Lógica interna para publicar un producto.
        fn _publicar(
            &mut self,
            vendedor: AccountId,
            nombre: String,
            descripcion: String,
            precio: Balance,
            stock: u32,
            categoria: String,
        ) -> Result<u32, Error> {
            let rol_vendedor = self.rol_de(vendedor)?;
            self.ensure(rol_vendedor.es_vendedor(), Error::SinPermiso)?;
            self.ensure(
                precio > 0
                    && stock > 0
                    && !nombre.is_empty()
                    && nombre.len() <= MAX_NOMBRE_LEN
                    && !descripcion.is_empty()
                    && descripcion.len() <= MAX_DESCRIPCION_LEN
                    && !categoria.is_empty()
                    && categoria.len() <= MAX_CATEGORIA_LEN,
                Error::ParamInvalido,
            )?;

            let pid = self.next_prod_id;
            self.next_prod_id = self.next_prod_id.checked_add(1).ok_or(Error::IdOverflow)?;

            let producto = Producto {
                vendedor,
                nombre,
                descripcion,
                precio,
                stock,
                categoria,
            };

            self.productos.insert(pid, &producto);
            Ok(pid)
        }

        /// Lógica interna para comprar un producto.
        fn _comprar(
            &mut self,
            comprador: AccountId,
            id_prod: u32,
            cant: u32,
        ) -> Result<u32, Error> {
            let rol_comprador = self.rol_de(comprador)?;
            self.ensure(rol_comprador.es_comprador(), Error::SinPermiso)?;
            self.ensure(cant > 0, Error::ParamInvalido)?;

            let mut producto = self.productos.get(id_prod).ok_or(Error::ProdInexistente)?;
            self.ensure(producto.vendedor != comprador, Error::AutoCompraProhibida)?;
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

            self.calificaciones.insert(
                oid,
                &CalificacionOrden {
                    comprador_califico: false,
                    vendedor_califico: false,
                },
            );

            Ok(oid)
        }

        /// Lógica interna para marcar una orden como enviada.
        fn _marcar_enviado(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            let mut orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;
            self.ensure(orden.vendedor == caller, Error::SinPermiso)?;

            if orden.estado == Estado::Cancelada {
                return Err(Error::OrdenCancelada);
            }
            self.ensure(orden.estado == Estado::Pendiente, Error::EstadoInvalido)?;

            orden.estado = Estado::Enviado;
            self.ordenes.insert(oid, &orden);
            Ok(())
        }

        /// Lógica interna para marcar una orden como recibida.
        fn _marcar_recibido(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            let mut orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;
            self.ensure(orden.comprador == caller, Error::SinPermiso)?;

            if orden.estado == Estado::Cancelada {
                return Err(Error::OrdenCancelada);
            }
            self.ensure(orden.estado == Estado::Enviado, Error::EstadoInvalido)?;

            orden.estado = Estado::Recibido;
            self.ordenes.insert(oid, &orden);
            self.cancelaciones_pendientes.remove(oid);

            Ok(())
        }

        /// Lógica interna para solicitar la cancelación de una orden.
        fn _solicitar_cancelacion(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            let mut orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;

            self.ensure(orden.estado != Estado::Cancelada, Error::OrdenCancelada)?;

            self.ensure(
                caller == orden.comprador || caller == orden.vendedor,
                Error::SinPermiso,
            )?;

            self.ensure(
                orden.estado == Estado::Pendiente || orden.estado == Estado::Enviado,
                Error::EstadoInvalido,
            )?;

            if orden.estado == Estado::Pendiente && caller == orden.comprador {
                let mut producto = self
                    .productos
                    .get(orden.id_prod)
                    .ok_or(Error::ProdInexistente)?;
                producto.stock = producto
                    .stock
                    .checked_add(orden.cantidad)
                    .ok_or(Error::StockOverflow)?;
                self.productos.insert(orden.id_prod, &producto);

                orden.estado = Estado::Cancelada;
                self.ordenes.insert(oid, &orden);
                self.cancelaciones_pendientes.remove(oid);

                return Ok(());
            }

            self.ensure(
                !self.cancelaciones_pendientes.contains(oid),
                Error::CancelacionYaPendiente,
            )?;

            self.cancelaciones_pendientes.insert(oid, &CancelacionPendiente {
                oid,
                solicitante: caller,
            });
            Ok(())
        }

        /// Lógica interna para aceptar la cancelación de una orden.
        fn _aceptar_cancelacion(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            let cancelacion = self
                .cancelaciones_pendientes
                .get(oid)
                .ok_or(Error::CancelacionInexistente)?;

            let orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;

            self.ensure(orden.estado != Estado::Cancelada, Error::OrdenCancelada)?;

            self.ensure(
                orden.estado == Estado::Pendiente || orden.estado == Estado::Enviado,
                Error::EstadoInvalido,
            )?;

            self.ensure(
                caller != cancelacion.solicitante,
                Error::SolicitanteCancelacion,
            )?;

            self.ensure(
                self.es_otro_participante(caller, &orden, cancelacion.solicitante),
                Error::SinPermiso,
            )?;

            let mut producto = self
                .productos
                .get(orden.id_prod)
                .ok_or(Error::ProdInexistente)?;
            producto.stock = producto
                .stock
                .checked_add(orden.cantidad)
                .ok_or(Error::StockOverflow)?;
            self.productos.insert(orden.id_prod, &producto);

            self.ordenes.insert(oid, &Orden {
                estado: Estado::Cancelada,
                ..orden
            });

            self.cancelaciones_pendientes.remove(oid);

            Ok(())
        }

        /// Lógica interna para rechazar la cancelación de una orden.
        fn _rechazar_cancelacion(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            let cancelacion = self
                .cancelaciones_pendientes
                .get(oid)
                .ok_or(Error::CancelacionInexistente)?;

            let orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;

            self.ensure(orden.estado != Estado::Cancelada, Error::OrdenCancelada)?;

            self.ensure(
                orden.estado == Estado::Pendiente || orden.estado == Estado::Enviado,
                Error::EstadoInvalido,
            )?;

            self.ensure(
                caller != cancelacion.solicitante,
                Error::SolicitanteCancelacion,
            )?;

            self.ensure(
                self.es_otro_participante(caller, &orden, cancelacion.solicitante),
                Error::SinPermiso,
            )?;

            self.cancelaciones_pendientes.remove(oid);

            Ok(())
        }

        /// Helper para validar condiciones.
        ///
        /// Esta función auxiliar facilita la validación de condiciones en el contrato,
        /// haciendo que el código sea más legible y expresivo.
        ///
        /// # Argumentos
        ///
        /// * `cond` - La condición booleana a verificar.
        /// * `err` - El error a devolver si la condición es falsa.
        ///
        /// # Retorno
        ///
        /// Devuelve `Ok(())` si la condición es verdadera, o `Err(err)` si es falsa.
        fn ensure(&self, cond: bool, err: Error) -> Result<(), Error> {
            if cond {
                Ok(())
            } else {
                Err(err)
            }
        }

        /// Helper que obtiene el rol de un usuario.
        ///
        /// # Argumentos
        ///
        /// * `quien` - La `AccountId` del usuario cuyo rol se desea obtener.
        ///
        /// # Errores
        ///
        /// Devuelve `Error::SinRegistro` si el usuario no está registrado.
        ///
        /// # Retorno
        ///
        /// Devuelve el `Rol` del usuario si está registrado.
        fn rol_de(&self, quien: AccountId) -> Result<Rol, Error> {
            self.roles.get(quien).ok_or(Error::SinRegistro)
        }

        /// Helper para validar que el caller sea el otro participante en una orden.
        ///
        /// Dado una orden y un solicitante, verifica que el caller sea el otro participante
        /// (comprador si el solicitante es vendedor, o vendedor si el solicitante es comprador).
        ///
        /// # Argumentos
        ///
        /// * `caller` - La `AccountId` de quien intenta aceptar/rechazar.
        /// * `orden` - La `Orden` en cuestión.
        /// * `solicitante` - La `AccountId` de quien solicitó la cancelación.
        ///
        /// # Retorno
        ///
        /// Devuelve `true` si el caller es el otro participante, `false` en caso contrario.
        fn es_otro_participante(
            &self,
            caller: AccountId,
            orden: &Orden,
            solicitante: AccountId,
        ) -> bool {
            (solicitante == orden.comprador && caller == orden.vendedor)
                || (solicitante == orden.vendedor && caller == orden.comprador)
        }

        /// Lógica interna para calificar al vendedor por el comprador.
        fn _calificar_vendedor(
            &mut self,
            caller: AccountId,
            oid: u32,
            puntos: u8,
        ) -> Result<(), Error> {
            let orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;

            self.ensure(orden.comprador == caller, Error::SinPermiso)?;
            self.ensure(orden.estado == Estado::Recibido, Error::OrdenNoRecibida)?;
            self.ensure(puntos >= 1 && puntos <= 5, Error::CalificacionInvalida)?;

            let mut calif = self.calificaciones.get(oid).unwrap_or(CalificacionOrden {
                comprador_califico: false,
                vendedor_califico: false,
            });
            self.ensure(!calif.comprador_califico, Error::YaCalificado)?;

            calif.comprador_califico = true;
            self.calificaciones.insert(oid, &calif);

            let mut rep = self
                .reputaciones
                .get(orden.vendedor)
                .unwrap_or(ReputacionUsuario {
                    como_comprador: (0, 0),
                    como_vendedor: (0, 0),
                });

            rep.como_vendedor.0 = rep
                .como_vendedor
                .0
                .checked_add(puntos as u32)
                .ok_or(Error::IdOverflow)?;
            rep.como_vendedor.1 = rep
                .como_vendedor
                .1
                .checked_add(1)
                .ok_or(Error::IdOverflow)?;

            self.reputaciones.insert(orden.vendedor, &rep);

            let producto = self
                .productos
                .get(orden.id_prod)
                .ok_or(Error::ProdInexistente)?;
            let mut cat_rep = self
                .calificaciones_por_categoria
                .get(producto.categoria.clone())
                .unwrap_or((0, 0));

            cat_rep.0 = cat_rep.0.checked_add(puntos as u32).ok_or(Error::IdOverflow)?;
            cat_rep.1 = cat_rep.1.checked_add(1).ok_or(Error::IdOverflow)?;
            self.calificaciones_por_categoria
                .insert(producto.categoria, &cat_rep);

            Ok(())
        }

        /// Lógica interna para calificar al comprador por el vendedor.
        fn _calificar_comprador(
            &mut self,
            caller: AccountId,
            oid: u32,
            puntos: u8,
        ) -> Result<(), Error> {
            let orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;

            self.ensure(orden.vendedor == caller, Error::SinPermiso)?;
            self.ensure(orden.estado == Estado::Recibido, Error::OrdenNoRecibida)?;
            self.ensure(puntos >= 1 && puntos <= 5, Error::CalificacionInvalida)?;

            let mut calif = self.calificaciones.get(oid).unwrap_or(CalificacionOrden {
                comprador_califico: false,
                vendedor_califico: false,
            });
            self.ensure(!calif.vendedor_califico, Error::YaCalificado)?;

            calif.vendedor_califico = true;
            self.calificaciones.insert(oid, &calif);

            let mut rep = self
                .reputaciones
                .get(orden.comprador)
                .unwrap_or(ReputacionUsuario {
                    como_comprador: (0, 0),
                    como_vendedor: (0, 0),
                });

            rep.como_comprador.0 = rep
                .como_comprador
                .0
                .checked_add(puntos as u32)
                .ok_or(Error::IdOverflow)?;
            rep.como_comprador.1 = rep
                .como_comprador
                .1
                .checked_add(1)
                .ok_or(Error::IdOverflow)?;

            self.reputaciones.insert(orden.comprador, &rep);

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        fn set_next_caller(caller: AccountId) {
            test::set_caller::<DefaultEnvironment>(caller);
        }

        fn get_accounts() -> test::DefaultAccounts<DefaultEnvironment> {
            test::default_accounts::<DefaultEnvironment>()
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
            let resultado =
                mp.publicar(nombre_largo, "Desc".to_string(), 100, 5, "Cat".to_string());
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
            let resultado = mp.comprar(pid, 3);

            assert_eq!(resultado, Ok(1));

            let producto = mp.obtener_producto(pid).unwrap();
            assert_eq!(producto.stock, 7);

            let orden = mp.obtener_orden(1).unwrap();
            assert_eq!(orden.comprador, accounts.bob);
            assert_eq!(orden.vendedor, accounts.alice);
            assert_eq!(orden.cantidad, 3);
            assert_eq!(orden.estado, Estado::Pendiente);
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
            mp.comprar(pid, 2).unwrap();
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
            let oid = mp.comprar(pid, 3).unwrap();

            assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));
        }

        /// Test: El comprador cancela unilateralmente una orden pendiente (restaura stock y marca cancelada).
        #[ink::test]
        fn comprador_cancela_unilateral_pendiente() {
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
            let oid = mp.comprar(pid, 3).unwrap();

            // Stock queda en 2 tras la compra.
            assert_eq!(mp.obtener_producto(pid).unwrap().stock, 2);
            assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Pendiente);

            // El comprador cancela en estado pendiente sin esperar al vendedor.
            assert_eq!(mp.solicitar_cancelacion(oid), Ok(()));

            let orden = mp.obtener_orden(oid).unwrap();
            assert_eq!(orden.estado, Estado::Cancelada);

            // Stock restaurado a 5 (stock original).
            let producto = mp.obtener_producto(pid).unwrap();
            assert_eq!(producto.stock, 5);

            // No debe quedar una solicitud pendiente que luego se acepte.
            assert_eq!(mp.aceptar_cancelacion(oid), Err(Error::CancelacionInexistente));
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
            mp.comprar(pid, 3).unwrap();
            assert_eq!(mp.obtener_producto(pid).unwrap().stock, 7);

            set_next_caller(accounts.charlie);
            mp.registrar(Rol::Comprador).unwrap();
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
            let oid1 = mp.comprar(pid1, 1).unwrap();
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
    }
}

// Re-exportaciones públicas para usar este contrato como dependencia
#[cfg(feature = "ink-as-dependency")]
pub use marketplace::{
    Estado, Marketplace, MarketplaceRef, Orden, Producto, ReputacionUsuario, Rol,
};
