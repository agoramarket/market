//! # Ágora Marketplace: Primera entrega
//!
//! La primera entrega de nuestro contrato se concentra en implementar
//!
//! * Control de acceso mediante _roles_ (`Comprador` / `Vendedor`).
//! * Publicación de productos con precio y stock.
//! * Flujo de compra y seguimiento de una orden (Pendiente → Enviado → Recibido).
//! * Gestión de errores expresiva y exhaustiva.
//!
//! Junto al contrato se incluyen pruebas de _unit testing_ (`#[ink::test]`) y
//! pruebas _end-to-end_ (`#[ink_e2e::test]`) que ilustran los escenarios
//! principales de uso.
//!
//! Todas las entradas públicas del contrato están debidamente documentadas con
//! Rustdoc para facilitar su comprensión y futura extensión.

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod marketplace {
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use bitflags::bitflags;
    use scale::{Decode, Encode};
    use scale_info::TypeInfo;

    // -------------------------------------------------------------------------
    // Roles
    // -------------------------------------------------------------------------

    bitflags! {
        /// Conjunto de permisos que un usuario puede poseer dentro del
        /// marketplace.  
        ///
        /// Este `bitflags` permite que un mismo `AccountId` sea simultáneamente
        /// comprador y vendedor si así lo desea.
        ///
        /// ```
        /// Rol::COMPRADOR | Rol::VENDEDOR   // usuario mixto
        /// Rol::COMPRADOR.is_empty()        // false
        /// ```
        #[derive(Encode, Decode, TypeInfo)]
        #[cfg_attr(feature = "std", derive(Debug))]
        pub struct Rol: u8 {
            /// Puede crear órdenes de compra.
            const COMPRADOR = 0b01;
            /// Puede publicar (vender) productos.
            const VENDEDOR  = 0b10;
        }
    }

    impl Default for Rol {
        /// Valor por defecto: sin permisos.
        fn default() -> Self {
            Self::empty()
        }
    }

    // -------------------------------------------------------------------------
    // Tipos de alto nivel
    // -------------------------------------------------------------------------

    /// Estados posibles de una orden de compra.
    #[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, TypeInfo)]
    pub enum Estado {
        /// El comprador ha pagado pero el vendedor aún no despachó la orden.
        Pendiente,
        /// El vendedor marcó la orden como enviada.
        Enviado,
        /// El comprador confirmó la recepción.
        Recibido,
    }

    /// Representa un producto ofrecido en el marketplace.
    #[derive(Encode, Decode, Clone, Debug, TypeInfo)]
    pub struct Producto {
        /// Cuenta del vendedor que lo publicó.
        vendedor: AccountId,
        /// Nombre legible (máx. 64 bytes).
        nombre:   String,
        /// Precio unitario en la moneda nativa de la cadena.
        precio:   Balance,
        /// Unidades disponibles para la venta.
        stock:    u32,
    }

    /// Representa una orden (compra) creada por un comprador.
    #[derive(Encode, Decode, Clone, Debug, TypeInfo)]
    pub struct Orden {
        /// Quien paga la orden.
        comprador: AccountId,
        /// Vendedor que la despacha.
        vendedor:  AccountId,
        /// ID del producto adquirido.
        id_prod:   u32,
        /// Unidades compradas.
        cantidad:  u32,
        /// Estado actual de la orden.
        estado:    Estado,
    }

    /// Errores que puede devolver el contrato.
    #[derive(Encode, Decode, Debug, PartialEq, Eq, TypeInfo)]
    pub enum Error {
        /// La cuenta ya se encuentra registrada.
        YaRegistrado,
        /// La cuenta no se encuentra registrada.
        SinRegistro,
        /// La cuenta no posee el rol requerido.
        SinPermiso,
        /// Parámetros inválidos (cero, cadena demasiado larga, etc.).
        ParamInvalido,
        /// El producto consultado no existe.
        ProdInexistente,
        /// Stock insuficiente para satisfacer la compra.
        StockInsuf,
        /// La orden consultada no existe.
        OrdenInexistente,
        /// Transición de estado no permitida.
        EstadoInvalido,
        /// Se alcanzó el máximo valor posible para IDs.
        IdOverflow,
    }

    // -------------------------------------------------------------------------
    // Almacenamiento del contrato
    // -------------------------------------------------------------------------

    /// Implementación principal del contrato.
    #[ink(storage)]
    pub struct Marketplace {
        /// Mapea cuentas a sus roles.
        roles: Mapping<AccountId, Rol>,
        /// Productos listados. Se indexan por un `u32` incremental.
        productos: Mapping<u32, Producto>,
        /// Órdenes emitidas. Se indexan por un `u32` incremental.
        ordenes: Mapping<u32, Orden>,
        /// Próximo ID libre para `Producto`.
        next_prod_id: u32,
        /// Próximo ID libre para `Orden`.
        next_order_id: u32,
    }

    impl Marketplace {
        // ---------------------------------------------------------------------
        // Constructor
        // ---------------------------------------------------------------------

        /// Instancia un contrato vacío.
        ///
        /// No asigna ningún rol; cada cuenta debe auto-registrarse mediante
        /// [`registrar`].
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

        // ---------------------------------------------------------------------
        // Auxiliares internas
        // ---------------------------------------------------------------------

        /// Valida una condición y retorna el error indicado si falla.
        ///
        /// Resulta útil para encadenar verificaciones de manera legible.
        fn ensure(&self, cond: bool, err: Error) -> Result<(), Error> {
            if cond {
                Ok(())
            } else {
                Err(err)
            }
        }

        /// Obtiene los roles de `who` o retorna `Error::SinRegistro`.
        fn rol(&self, who: AccountId) -> Result<Rol, Error> {
            self.roles.get(who).ok_or(Error::SinRegistro)
        }

        /// Garantiza que `who` posea al menos los bits de `rol`.
        fn assert_rol(&self, who: AccountId, rol: Rol) -> Result<(), Error> {
            self.ensure(self.rol(who)?.contains(rol), Error::SinPermiso)
        }

        // ---------------------------------------------------------------------
        // Entradas públicas (mensajes)
        // ---------------------------------------------------------------------

        /// Registra al _caller_ con los roles indicados.
        ///
        /// Requisitos:
        /// * El `caller` **no** debe estar registrado previamente.
        /// * `roles` no puede ser vacío.
        #[ink(message)]
        pub fn registrar(&mut self, roles: Rol) -> Result<(), Error> {
            let caller = self.env().caller();
            self.ensure(!self.roles.contains(caller), Error::YaRegistrado)?;
            self.ensure(!roles.is_empty(), Error::ParamInvalido)?;
            self.roles.insert(caller, &roles);
            Ok(())
        }

        /// Publica un nuevo producto y devuelve su `id`.
        ///
        /// Requisitos:
        /// * El `caller` debe ser `Rol::VENDEDOR`.
        /// * `precio` y `stock` > 0.
        /// * `nombre.len() <= 64`.
        #[ink(message)]
        pub fn publicar(
            &mut self,
            nombre: String,
            precio: Balance,
            stock:  u32,
        ) -> Result<u32, Error> {
            let vendedor = self.env().caller();
            self.assert_rol(vendedor, Rol::VENDEDOR)?;
            self.ensure(precio > 0 && stock > 0, Error::ParamInvalido)?;
            self.ensure(nombre.len() <= 64, Error::ParamInvalido)?;

            let pid = self.next_prod_id;
            self.next_prod_id = self.next_prod_id.checked_add(1).ok_or(Error::IdOverflow)?;

            self.productos.insert(
                pid,
                &Producto { vendedor, nombre, precio, stock },
            );
            Ok(pid)
        }

        // ─────────────────────────────────────────────────────────────────────
        // Compras
        // ─────────────────────────────────────────────────────────────────────

        /// Crea una orden de compra y devuelve su `id`.
        ///
        /// Requisitos:
        /// 1. El `caller` debe tener el rol [`Rol::COMPRADOR`].
        /// 2. `cant` > `0`.
        /// 3. El producto identificado por `id_prod` debe existir.
        /// 4. Debe haber `stock` suficiente.
        ///
        /// Efectos secundarios:
        /// * Se debita el `stock` del producto.
        /// * Se persiste una nueva entrada en `ordenes`.
        ///
        /// # Ejemplo
        /// ```
        /// # use marketplace::{Marketplace, Rol};
        /// # use ink::env::test;
        /// # let mut mp = Marketplace::new();
        /// # let alice = test::default_accounts::<ink::env::DefaultEnvironment>().alice;
        /// # let bob   = test::default_accounts::<ink::env::DefaultEnvironment>().bob;
        /// test::set_caller::<ink::env::DefaultEnvironment>(alice);
        /// mp.registrar(Rol::VENDEDOR).unwrap();
        /// let pid = mp.publicar("te".into(), 10, 3).unwrap();
        ///
        /// test::set_caller::<ink::env::DefaultEnvironment>(bob);
        /// mp.registrar(Rol::COMPRADOR).unwrap();
        /// let oid = mp.comprar(pid, 1).unwrap();
        /// assert_eq!(mp.get_orden(oid).unwrap().cantidad, 1);
        /// ```
        #[ink(message)]
        pub fn comprar(&mut self, id_prod: u32, cant: u32) -> Result<u32, Error> {
            let comprador = self.env().caller();
            self.assert_rol(comprador, Rol::COMPRADOR)?;
            self.ensure(cant > 0, Error::ParamInvalido)?;

            // Obtiene el producto ó falla si no existe.
            let mut p = self.productos.get(id_prod).ok_or(Error::ProdInexistente)?;

            // Stock disponible.
            self.ensure(p.stock >= cant, Error::StockInsuf)?;
            p.stock -= cant;
            self.productos.insert(id_prod, &p);

            // Generación de ID de orden con overflow seguro.
            let oid = self.next_order_id;
            self.next_order_id = self
                .next_order_id
                .checked_add(1)
                .ok_or(Error::IdOverflow)?;

            // Persiste la orden.
            self.ordenes.insert(
                oid,
                &Orden {
                    comprador,
                    vendedor: p.vendedor,
                    id_prod,
                    cantidad: cant,
                    estado: Estado::Pendiente,
                },
            );
            Ok(oid)
        }

        // ─────────────────────────────────────────────────────────────────────
        // Seguimiento de estado de órdenes
        // ─────────────────────────────────────────────────────────────────────

        /// El vendedor marca la orden como **Enviada**.
        ///
        /// Requisitos:
        /// * `caller` debe ser el `vendedor` de la orden.
        /// * Estado actual debe ser [`Estado::Pendiente`].
        #[ink(message)]
        pub fn marcar_enviado(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            let mut o = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;
            self.ensure(o.vendedor == caller, Error::SinPermiso)?;
            self.ensure(o.estado == Estado::Pendiente, Error::EstadoInvalido)?;
            o.estado = Estado::Enviado;
            self.ordenes.insert(oid, &o);
            Ok(())
        }

        /// El comprador confirma que recibió la orden.
        ///
        /// Requisitos:
        /// * `caller` debe ser el `comprador` de la orden.
        /// * Estado actual debe ser [`Estado::Enviado`].
        #[ink(message)]
        pub fn marcar_recibido(&mut self, oid: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            let mut o = self.ordenes.get(oid).ok_or(Error::OrdenInexistente)?;
            self.ensure(o.comprador == caller, Error::SinPermiso)?;
            self.ensure(o.estado == Estado::Enviado, Error::EstadoInvalido)?;
            o.estado = Estado::Recibido;
            self.ordenes.insert(oid, &o);
            Ok(())
        }

        // ─────────────────────────────────────────────────────────────────────
        // Lectura de estado (consultas)
        // ─────────────────────────────────────────────────────────────────────

        /// Devuelve el [`Producto`] asociado al identificador dado, si existe.
        #[ink(message)]
        pub fn get_producto(&self, id: u32) -> Option<Producto> {
            self.productos.get(id)
        }

        /// Devuelve la [`Orden`] asociada al identificador dado, si existe.
        #[ink(message)]
        pub fn get_orden(&self, id: u32) -> Option<Orden> {
            self.ordenes.get(id)
        }

        /// Devuelve los [`Rol`]es asignados a `user`, si la cuenta está
        /// registrada.
        #[ink(message)]
        pub fn get_roles(&self, user: AccountId) -> Option<Rol> {
            self.roles.get(user)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pruebas unitarias
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    //! Conjunto de pruebas unitarias que validan:
    //! * El _happy path_ completo de compra-venta.
    //! * Manejo de errores en registro y permisos.
    //! * Falta de stock.
    //! * Transiciones de estado inválidas.

    use super::*;
    use ink::env::{test, DefaultEnvironment};

    type AccountId = <DefaultEnvironment as ink::env::Environment>::AccountId;
    type Balance   = <DefaultEnvironment as ink::env::Environment>::Balance;

    /// Retorna tres cuentas de testing (Alice, Bob y Charlie).
    fn accounts() -> (AccountId, AccountId, AccountId) {
        let accounts = test::default_accounts::<DefaultEnvironment>();
        (accounts.alice, accounts.bob, accounts.charlie)
    }

    /// Cambia el `caller` para la próxima llamada de contrato dentro del
    /// entorno de pruebas.
    fn set_caller(caller: AccountId) {
        test::set_caller::<DefaultEnvironment>(caller);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Happy path
    // ─────────────────────────────────────────────────────────────────────────

    /// Recorre el flujo exitoso completo:
    ///
    /// 1. Vendedor y comprador se registran.
    /// 2. El vendedor publica un producto.
    /// 3. El comprador lo compra.
    /// 4. El vendedor lo marca como enviado.
    /// 5. El comprador confirma la recepción.
    #[ink::test]
    fn flujo_feliz() {
        let mut mp = Marketplace::new();
        let (vendedor, comprador, _) = accounts();

        // Registro de usuarios.
        set_caller(vendedor);
        assert_eq!(mp.registrar(Rol::VENDEDOR), Ok(()));
        set_caller(comprador);
        assert_eq!(mp.registrar(Rol::COMPRADOR), Ok(()));

        // Publicación de producto.
        set_caller(vendedor);
        let pid = mp.publicar("mate".into(), 10, 5).unwrap();
        assert_eq!(mp.get_producto(pid).unwrap().stock, 5);

        // Compra.
        set_caller(comprador);
        let oid = mp.comprar(pid, 2).unwrap();
        assert_eq!(mp.get_orden(oid).unwrap().estado, Estado::Pendiente);
        assert_eq!(mp.get_producto(pid).unwrap().stock, 3);

        // Envío y recepción.
        set_caller(vendedor);
        assert_eq!(mp.marcar_enviado(oid), Ok(()));
        assert_eq!(mp.get_orden(oid).unwrap().estado, Estado::Enviado);

        set_caller(comprador);
        assert_eq!(mp.marcar_recibido(oid), Ok(()));
        assert_eq!(mp.get_orden(oid).unwrap().estado, Estado::Recibido);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Casos de error
    // ─────────────────────────────────────────────────────────────────────────

    /// Valida fallas de registro duplicado y falta de permisos.
    #[ink::test]
    fn errores_de_registro_y_roles() {
        let mut mp = Marketplace::new();
        let (usuario, _, _) = accounts();

        set_caller(usuario);
        // `roles` vacío.
        assert_eq!(mp.registrar(Rol::empty()), Err(Error::ParamInvalido));
        // Registro OK.
        assert_eq!(mp.registrar(Rol::VENDEDOR), Ok(()));
        // Doble registro.
        assert_eq!(mp.registrar(Rol::COMPRADOR), Err(Error::YaRegistrado));

        // Otro usuario sin rol de vendedor intenta publicar.
        let otro = test::default_accounts::<DefaultEnvironment>().bob;
        set_caller(otro);
        mp.registrar(Rol::COMPRADOR).unwrap();
        assert_eq!(
            mp.publicar("falso".into(), 10, 1),
            Err(Error::SinPermiso)
        );
    }

    /// Rechaza compras con cantidad 0 o mayor al stock.
    #[ink::test]
    fn stock_insuficiente() {
        let mut mp = Marketplace::new();
        let (vendedor, comprador, _) = accounts();

        set_caller(vendedor);
        mp.registrar(Rol::VENDEDOR).unwrap();
        let pid = mp.publicar("libro".into(), 20, 1).unwrap();

        set_caller(comprador);
        mp.registrar(Rol::COMPRADOR).unwrap();
        assert_eq!(mp.comprar(pid, 0), Err(Error::ParamInvalido));
        assert_eq!(mp.comprar(pid, 2), Err(Error::StockInsuf));
    }

    /// Verifica transiciones de estado no permitidas y violaciones de autoría.
    #[ink::test]
    fn transiciones_invalidas() {
        let mut mp = Marketplace::new();
        let (vendedor, comprador, intruso) = accounts();

        set_caller(vendedor);
        mp.registrar(Rol::VENDEDOR).unwrap();
        let pid = mp.publicar("pc".into(), 100, 1).unwrap();

        set_caller(comprador);
        mp.registrar(Rol::COMPRADOR).unwrap();
        let oid = mp.comprar(pid, 1).unwrap();

        // El comprador no puede marcar como recibido si no fue enviado.
        assert_eq!(mp.marcar_recibido(oid), Err(Error::EstadoInvalido));

        // Un tercero no puede marcar la orden como enviada.
        set_caller(intruso);
        assert_eq!(mp.marcar_enviado(oid), Err(Error::SinPermiso));

        // El vendedor la marca como enviada.
        set_caller(vendedor);
        mp.marcar_enviado(oid).unwrap();

        // No puede volver a marcarla como enviada.
        assert_eq!(mp.marcar_enviado(oid), Err(Error::EstadoInvalido));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pruebas end-to-end (cargo feature `e2e`)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "e2e"))]
mod e2e_tests {
    //! Prueba completa desplegando el contrato en un nodo Substrate de prueba,
    //! utilizando el `ink_e2e` framework.  
    //!
    //! Se replica el flujo exitoso documentado en [`tests::flujo_feliz`].

    use super::marketplace::{MarketplaceRef, Rol};
    use ink_e2e::{build_message, PolkadotConfig};

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    /// Escenario “happy path” en entorno real.
    #[ink_e2e::test]
    async fn flujo_completo_e2e(
        mut client: ink_e2e::Client<PolkadotConfig>,
    ) -> E2EResult<()> {
        let constructor = MarketplaceRef::new();
        let vendedor   = ink_e2e::account_key("Alice");
        let comprador  = ink_e2e::account_key("Bob");

        // Despliegue del contrato.
        let contract = client
            .instantiate("marketplace", &vendedor, constructor, 0, None)
            .await?
            .account_id;

        // Registro de roles.
        let msg = build_message::<MarketplaceRef>(contract.clone())
            .call(|c| c.registrar(Rol::VENDEDOR));
        client.call(&vendedor, msg, 0, None).await?.assert_ok()?;

        let msg = build_message::<MarketplaceRef>(contract.clone())
            .call(|c| c.registrar(Rol::COMPRADOR));
        client.call(&comprador, msg, 0, None).await?.assert_ok()?;

        // Publicación y compra.
        let msg = build_message::<MarketplaceRef>(contract.clone())
            .call(|c| c.publicar("auto".into(), 1_000_000, 3));
        let pid: u32 =
            client.call(&vendedor, msg, 0, None).await?.return_value()?;

        let msg = build_message::<MarketplaceRef>(contract.clone())
            .call(|c| c.comprar(pid, 2));
        let oid: u32 =
            client.call(&comprador, msg, 0, None).await?.return_value()?;

        // Envío y recepción.
        let msg = build_message::<MarketplaceRef>(contract.clone())
            .call(|c| c.marcar_enviado(oid));
        client.call(&vendedor, msg, 0, None).await?.assert_ok()?;

        let msg = build_message::<MarketplaceRef>(contract.clone())
            .call(|c| c.marcar_recibido(oid));
        client.call(&comprador, msg, 0, None).await?.assert_ok()?;

        Ok(())
    }
}