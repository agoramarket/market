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
            self.ensure(orden.comprador == caller || orden.vendedor == caller, Error::SinPermiso)?;
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
        pub fn listar_ordenes_de_comprador(&self) -> Vec<Orden> {
            let caller = self.env().caller();
            self._listar_ordenes_de_comprador(caller)
        }

        // A partir de acá están las funciones internas que implementan la lógica del contrato.

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
                precio > 0 && stock > 0 
                && !nombre.is_empty() && nombre.len() <= 64 
                && !descripcion.is_empty() && descripcion.len() <= 256 
                && !categoria.is_empty() && categoria.len() <= 32,
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
            self.ensure(producto.vendedor != comprador, Error::SinPermiso)?;
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

        /// Lógica interna para marcar una orden como enviada.
        fn _marcar_enviado(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            let mut orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;
            self.ensure(orden.vendedor == caller, Error::SinPermiso)?;
            self.ensure(orden.estado == Estado::Pendiente, Error::EstadoInvalido)?;

            orden.estado = Estado::Enviado;
            self.ordenes.insert(oid, &orden);
            Ok(())
        }

        /// Lógica interna para marcar una orden como recibida.
        fn _marcar_recibido(&mut self, caller: AccountId, oid: u32) -> Result<(), Error> {
            let mut orden = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;
            self.ensure(orden.comprador == caller, Error::SinPermiso)?;
            self.ensure(orden.estado == Estado::Enviado, Error::EstadoInvalido)?;

            orden.estado = Estado::Recibido;
            self.ordenes.insert(oid, &orden);
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
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{test, DefaultEnvironment};

        // --- HELPERS ---
        fn set_next_caller(caller: AccountId) {
            test::set_caller::<DefaultEnvironment>(caller);
        }

        fn get_accounts() -> test::DefaultAccounts<DefaultEnvironment> {
            test::default_accounts::<DefaultEnvironment>()
        }

        // ===== TESTS DE REGISTRO =====

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

        // ===== TESTS DE MODIFICACIÓN DE ROL =====

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

        // ===== TESTS DE PUBLICACIÓN DE PRODUCTOS =====

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
            let resultado = mp.publicar(
                nombre_largo,
                "Desc".to_string(),
                100,
                5,
                "Cat".to_string(),
            );
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

        // ===== TESTS DE LISTADO DE PRODUCTOS =====

        /// Test: Listar productos de un vendedor.
        #[ink::test]
        fn listar_productos_de_vendedor() {
            let accounts = get_accounts();
            let mut mp = Marketplace::new();

            set_next_caller(accounts.alice);
            mp.registrar(Rol::Vendedor).unwrap();

            mp.publicar("Producto1".to_string(), "Desc1".to_string(), 100, 5, "Cat1".to_string()).unwrap();
            mp.publicar("Producto2".to_string(), "Desc2".to_string(), 200, 10, "Cat2".to_string()).unwrap();

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

        // ===== TESTS DE COMPRA =====

        /// Test: Compra exitosa de producto.
        #[ink::test]
        fn comprar_producto_exitoso() {
            let accounts = get_accounts();
            let mut mp = Marketplace::new();

            set_next_caller(accounts.alice);
            mp.registrar(Rol::Vendedor).unwrap();
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

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
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

            let resultado = mp.comprar(pid, 1);
            assert_eq!(resultado, Err(Error::SinPermiso));
        }

        /// Test: Error al comprar sin estar registrado.
        #[ink::test]
        fn comprar_sin_registro() {
            let accounts = get_accounts();
            let mut mp = Marketplace::new();

            set_next_caller(accounts.alice);
            mp.registrar(Rol::Vendedor).unwrap();
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

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
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

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
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 5, "Cat".to_string()).unwrap();

            set_next_caller(accounts.bob);
            mp.registrar(Rol::Comprador).unwrap();
            let resultado = mp.comprar(pid, 10);
            assert_eq!(resultado, Err(Error::StockInsuf));
        }

        // ===== TESTS DE LISTADO DE ÓRDENES =====

        /// Test: Listar órdenes del comprador que llama.
        #[ink::test]
        fn listar_ordenes_de_comprador() {
            let accounts = get_accounts();
            let mut mp = Marketplace::new();

            set_next_caller(accounts.alice);
            mp.registrar(Rol::Vendedor).unwrap();
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

            set_next_caller(accounts.bob);
            mp.registrar(Rol::Comprador).unwrap();
            mp.comprar(pid, 2).unwrap();
            mp.comprar(pid, 3).unwrap();

            let ordenes = mp.listar_ordenes_de_comprador();
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

            let ordenes = mp.listar_ordenes_de_comprador();
            assert_eq!(ordenes.len(), 0);
        }

        // ===== TESTS DE FLUJO DE ÓRDENES =====

        /// Test: Marcar orden como enviada exitosamente.
        #[ink::test]
        fn marcar_orden_enviado_exitoso() {
            let accounts = get_accounts();
            let mut mp = Marketplace::new();

            set_next_caller(accounts.alice);
            mp.registrar(Rol::Vendedor).unwrap();
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

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
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

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
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

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
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

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
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

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
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

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

        // ===== TESTS DE OVERFLOW =====

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
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 5, "Cat".to_string()).unwrap();

            set_next_caller(accounts.bob);
            mp.registrar(Rol::Comprador).unwrap();

            mp.next_order_id = u32::MAX;
            assert_eq!(mp.comprar(pid, 1), Err(Error::IdOverflow));
        }

        // ===== TESTS DE ROL AMBOS =====

        /// Test: Usuario con rol Ambos puede comprar productos de otros vendedores.
        #[ink::test]
        fn rol_ambos_puede_comprar_y_vender() {
            let accounts = get_accounts();
            let mut mp = Marketplace::new();

            set_next_caller(accounts.alice);
            mp.registrar(Rol::Ambos).unwrap();
            let _pid_alice = mp.publicar("Test Alice".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

            set_next_caller(accounts.bob);
            mp.registrar(Rol::Ambos).unwrap();
            let pid_bob = mp.publicar("Test Bob".to_string(), "Desc".to_string(), 50, 5, "Cat".to_string()).unwrap();

            // Alice compra el producto de Bob
            set_next_caller(accounts.alice);
            let oid = mp.comprar(pid_bob, 2).unwrap();
            assert_eq!(oid, 1);

            let producto = mp.obtener_producto(pid_bob).unwrap();
            assert_eq!(producto.stock, 3);
        }

        /// Test: Error al intentar obtener orden sin ser comprador ni vendedor.
        #[ink::test]
        fn obtener_orden_sin_permiso() {
            let accounts = get_accounts();
            let mut mp = Marketplace::new();

            set_next_caller(accounts.alice);
            mp.registrar(Rol::Vendedor).unwrap();
            let pid = mp.publicar("Test".to_string(), "Desc".to_string(), 100, 10, "Cat".to_string()).unwrap();

            set_next_caller(accounts.bob);
            mp.registrar(Rol::Comprador).unwrap();
            let oid = mp.comprar(pid, 1).unwrap();

            // Charlie intenta acceder a la orden de Alice y Bob
            set_next_caller(accounts.charlie);
            assert_eq!(mp.obtener_orden(oid), Err(Error::SinPermiso));
        }
    }
}