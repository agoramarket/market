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

        /// Lista todas las órdenes realizadas por un comprador específico.
        ///
        /// # Argumentos
        ///
        /// * `comprador` - La `AccountId` del comprador cuyas órdenes se desean listar.
        ///
        /// # Retorno
        ///
        /// Devuelve un `Vec<Orden>` con todas las órdenes del comprador.
        /// Si el comprador no tiene órdenes, devuelve un vector vacío.
        ///
        /// # Nota
        ///
        /// Esta función itera sobre todos los IDs de órdenes, por lo que su costo
        /// aumenta linealmente con el número total de órdenes en el marketplace.
        #[ink(message)]
        pub fn listar_ordenes_de_comprador(&self, comprador: AccountId) -> Vec<Orden> {
            self._listar_ordenes_de_comprador(comprador)
        }

        // A partir de acá están las funciones internas que implementan la lógica del contrato.

        /// Lógica interna para listar productos de un vendedor.
        fn _listar_productos_de_vendedor(&self, vendedor: AccountId) -> Vec<Producto> {
            let mut productos_vendedor = Vec::new();

            // Itera sobre todos los IDs de productos desde 1 hasta el último ID generado.
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

            // Itera sobre todos los IDs de órdenes desde 1 hasta el último ID generado.
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
            // Asegura que el usuario (caller) no esté ya registrado con un rol. Si ya está registrado, devuelve `Error::YaRegistrado`.
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
            // En un inicio fué implementado con saturating_add, pero luego refactoricé porque S_A "silencia" el error de desbordamiento y "congela" _next_prod_id_ en u32::MAX.
            // En ese escenario hipotético, el contrato crearía productos con el mismo ID.
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
            // Si la condición es verdadera, devuelve `Ok`.
            if cond {
                Ok(())
            } else {
                // Si la condición es falsa, devuelve el error especificado.
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
            // Intenta obtener el rol del usuario. Si no existe, devuelve `Error::SinRegistro`.
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

        /// Test para el flujo completo y casos de éxito.
        /// 1. Registra un comprador y un vendedor.
        /// 2. El vendedor publica un producto.
        /// 3. El comprador adquiere el producto, creando una orden.
        /// 4. El vendedor marca la orden como enviada.
        /// 5. El comprador marca la orden como recibida.
        #[ink::test]
        fn test_flujo_completo_exitoso() {
            let accounts = get_accounts();
            let (vendedor_acc, comprador_acc) = (accounts.alice, accounts.bob);
            let mut mp = Marketplace::new();

            // 1. Registro
            set_next_caller(vendedor_acc);
            assert_eq!(mp.registrar(Rol::Vendedor), Ok(()));
            assert_eq!(mp.obtener_rol(vendedor_acc), Some(Rol::Vendedor));

            set_next_caller(comprador_acc);
            assert_eq!(mp.registrar(Rol::Comprador), Ok(()));
            assert_eq!(mp.obtener_rol(comprador_acc), Some(Rol::Comprador));

            // 2. Publicación de producto
            set_next_caller(vendedor_acc);
            let res_pub = mp.publicar("Test Product".to_string(), 100, 10);
            assert_eq!(res_pub, Ok(1));
            let pid = res_pub.unwrap();
            let prod = mp.obtener_producto(pid).unwrap();
            assert_eq!(prod.vendedor, vendedor_acc);
            assert_eq!(prod.stock, 10);

            // 3. Compra
            set_next_caller(comprador_acc);
            let res_compra = mp.comprar(pid, 5);
            assert_eq!(res_compra, Ok(1));
            let oid = res_compra.unwrap();
            let prod_actualizado = mp.obtener_producto(pid).unwrap();
            assert_eq!(prod_actualizado.stock, 5); // Verifica reducción de stock

            let orden = mp.obtener_orden(oid).unwrap();
            assert_eq!(orden.comprador, comprador_acc);
            assert_eq!(orden.vendedor, vendedor_acc);
            assert_eq!(orden.estado, Estado::Pendiente);

            // 4. Marcar como enviado
            set_next_caller(vendedor_acc);
            assert_eq!(mp.marcar_enviado(oid), Ok(()));
            assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Enviado);

            // 5. Marcar como recibido
            set_next_caller(comprador_acc);
            assert_eq!(mp.marcar_recibido(oid), Ok(()));
            assert_eq!(mp.obtener_orden(oid).unwrap().estado, Estado::Recibido);
        }

        /// Test para verificar los errores de permisos y de parámetros inválidos.
        /// - Intenta registrar un usuario ya registrado.
        /// - Intenta publicar/comprar sin el rol adecuado o sin registro.
        /// - Intenta publicar con parámetros inválidos.
        /// - Intenta comprar más stock del disponible o un producto inexistente.
        #[ink::test]
        fn test_errores_permisos_y_parametros() {
            let accounts = get_accounts();
            let (vendedor_acc, comprador_acc, sin_rol_acc) =
                (accounts.alice, accounts.bob, accounts.charlie);
            let mut mp = Marketplace::new();

            // Registro
            set_next_caller(vendedor_acc);
            mp.registrar(Rol::Vendedor).unwrap();
            assert_eq!(mp.registrar(Rol::Vendedor), Err(Error::YaRegistrado));

            set_next_caller(comprador_acc);
            mp.registrar(Rol::Comprador).unwrap();

            // Errores de publicación
            set_next_caller(comprador_acc); // Un comprador no puede publicar
            assert_eq!(
                mp.publicar("Fail".to_string(), 1, 1),
                Err(Error::SinPermiso)
            );
            set_next_caller(sin_rol_acc); // Un usuario sin rol no puede publicar
            assert_eq!(
                mp.publicar("Fail".to_string(), 1, 1),
                Err(Error::SinRegistro)
            );
            set_next_caller(vendedor_acc); // Vendedor con parámetros inválidos
            assert_eq!(
                mp.publicar("Fail".to_string(), 0, 1),
                Err(Error::ParamInvalido)
            ); // Precio 0
            assert_eq!(
                mp.publicar("Fail".to_string(), 1, 0),
                Err(Error::ParamInvalido)
            ); // Stock 0

            // Publicación válida para pruebas de compra
            let pid = mp.publicar("Test".to_string(), 10, 5).unwrap();

            // Errores de compra
            set_next_caller(vendedor_acc); // Un vendedor no puede comprar
            assert_eq!(mp.comprar(pid, 1), Err(Error::SinPermiso));
            set_next_caller(sin_rol_acc); // Sin rol no puede comprar
            assert_eq!(mp.comprar(pid, 1), Err(Error::SinRegistro));
            set_next_caller(comprador_acc); // Comprador con parámetros/estado inválido
            assert_eq!(mp.comprar(99, 1), Err(Error::ProdInexistente)); // Producto no existe
            assert_eq!(mp.comprar(pid, 0), Err(Error::ParamInvalido)); // Cantidad 0
            assert_eq!(mp.comprar(pid, 10), Err(Error::StockInsuf)); // Stock insuficiente
        }

        /// Test para verificar la lógica de cambio de estado de las órdenes.
        /// - Intenta marcar una orden como enviada/recibida por la persona incorrecta.
        /// - Intenta cambiar el estado de una orden en un orden incorrecto (e.g., Recibido antes de Enviado).
        #[ink::test]
        fn test_errores_flujo_de_orden() {
            let accounts = get_accounts();
            let (vendedor_acc, comprador_acc, otro_acc) =
                (accounts.alice, accounts.bob, accounts.charlie);
            let mut mp = Marketplace::new();

            // Setup: Vendedor, Comprador, Producto, Orden
            set_next_caller(vendedor_acc);
            mp.registrar(Rol::Vendedor).unwrap();
            let pid = mp.publicar("Test".to_string(), 10, 5).unwrap();
            set_next_caller(comprador_acc);
            mp.registrar(Rol::Comprador).unwrap();
            let oid = mp.comprar(pid, 2).unwrap();

            // Errores al marcar como enviado
            set_next_caller(comprador_acc); // Comprador no puede marcar enviado
            assert_eq!(mp.marcar_enviado(oid), Err(Error::SinPermiso));
            set_next_caller(otro_acc); // Otro usuario no puede
            assert_eq!(mp.marcar_enviado(oid), Err(Error::SinPermiso));
            set_next_caller(vendedor_acc); // El vendedor correcto, pero con un ID de orden inexistente
            assert_eq!(mp.marcar_enviado(99), Err(Error::OrdenInexistente));

            // Errores al marcar como recibido
            set_next_caller(vendedor_acc); // Vendedor no puede marcar recibido
            assert_eq!(mp.marcar_recibido(oid), Err(Error::SinPermiso));
            // No se puede marcar recibido si no está enviado
            set_next_caller(comprador_acc);
            assert_eq!(mp.marcar_recibido(oid), Err(Error::EstadoInvalido));

            // Flujo correcto para probar más errores
            set_next_caller(vendedor_acc);
            mp.marcar_enviado(oid).unwrap(); // Ahora está Enviado

            // No se puede marcar enviado de nuevo
            assert_eq!(mp.marcar_enviado(oid), Err(Error::EstadoInvalido));

            set_next_caller(comprador_acc);
            mp.marcar_recibido(oid).unwrap(); // Ahora está Recibido

            // No se puede marcar recibido de nuevo
            assert_eq!(mp.marcar_recibido(oid), Err(Error::EstadoInvalido));
        }

        /// Test para el manejo de desbordamiento de IDs.
        /// Se simula un estado donde los contadores de IDs están al máximo valor de u32
        /// y se verifica que el contrato devuelva `Error::IdOverflow`.
        #[ink::test]
        fn test_overflow_ids() {
            let accounts = get_accounts();
            let mut mp = Marketplace::new();

            set_next_caller(accounts.alice);
            mp.registrar(Rol::Vendedor).unwrap();

            // Simula overflow de ID de producto
            mp.next_prod_id = u32::MAX;
            assert_eq!(
                mp.publicar("Overflow Prod".to_string(), 1, 1),
                Err(Error::IdOverflow)
            );

            // Resetea para probar overflow de orden
            mp.next_prod_id = 1;
            let pid = mp.publicar("Test Prod".to_string(), 1, 1).unwrap();

            set_next_caller(accounts.bob);
            mp.registrar(Rol::Comprador).unwrap();

            // Simula overflow de ID de orden
            mp.next_order_id = u32::MAX;
            assert_eq!(mp.comprar(pid, 1), Err(Error::IdOverflow));
        }
    }
}
