#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod market {

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    use ink::storage::Mapping;
    //use ink::storage::StorageVec;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
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

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug,PartialEq, Eq, Clone, Copy, Default)]
    pub enum Rol {
        #[default]
        Comprador,
        Vendedor,
    }

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug,PartialEq, Eq, Clone)]

    pub struct Usuario{
        nombre: String,
        // /// Dirección del usuario
        rol: Rol,
        reputacion: u8,
    }
    
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug,PartialEq, Eq, Clone)]
    pub struct Producto {
        id: u32, // Identificador único del producto
        nombre: String,
        descripcion: String,
        precio: Balance,
        cantidad: u32,
        categoria: String,
        vendedor: AccountId,
    }

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug,PartialEq, Eq, Clone)]
    enum EstadoOrden {
        Pendiente,
        Enviado,
        Recibido,
        Cancelada,
    }

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout)
    )]
    #[derive(Debug,PartialEq, Eq, Clone)]
    struct Orden {
        comprador: Usuario,
        vendedor: Usuario,
        producto_nombre: String,
        cantidad: u32,
        estado: EstadoOrden,
    }

    //atributo storage: indica que esta struct se usará para almacenar datos en la cadena de bloques
    #[ink(storage)]
    pub struct Market {
        /// Stores a single `bool` value on the storage.
        value: bool,
        owner: AccountId,
        usuarios: Mapping<AccountId, Usuario>,
        productos: Vec<Producto>,
        siguiente_id_producto: u32,
        ordenes: Vec<Orden>,
    }

    impl Market {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { 
                value: init_value,
                owner: Self::env().caller(), // Almacena el creador del contrato como owner
                usuarios: Mapping::new(),
                productos: Vec::new(),
                siguiente_id_producto: 1,
                ordenes: Vec::new(),
            }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        /// 
        /// constructor por defecto: indica que esta función se ejecuta al desplegar el contrato
        /// que es desplegar un contrato? 
        /// En el contexto de contratos inteligentes, desplegar un contrato significa crear una instancia del contrato en la cadena de bloques.
        /// Cuando se despliega un contrato, se crea un nuevo espacio de almacenamiento en la cadena de bloques para ese contrato, y se ejecuta su constructor para inicializar el estado del contrato.
        /// En este caso, el constructor por defecto inicializa el valor de `bool` a `false`, lo que significa que al desplegar el contrato, el valor inicial de `value` será `false`.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(false)
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        /// 
        /// interfaz publica: indica que esta función puede ser llamada por los usuarios
        /// atributo ink::message: indica que esta función puede ser llamada por los usuarios, puede ser invocada por transacciones
        #[ink(message)] 
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }

        /// Registra un nuevo usuario en el sistema.
        #[ink(message)]
        pub fn registrar_usuario(&mut self, nombre:String, rol: Rol) {
            let caller = self.env().caller();
            let usuario = Usuario {
                nombre,
                rol,
                reputacion: 0, // Inicialmente la reputación es 0
            };
            
            self.usuarios.insert(caller, &usuario);
            
        }

        #[ink(message)]
        pub fn get_owner(&self) -> AccountId {
            self.env().caller()
        }

        #[ink(message)]
        pub fn publicar_producto(&mut self, nombre: String, descripcion: String, precio: Balance, cantidad: u32, categoria: String)-> Result<(), Error> {
            
            let caller = self.env().caller();
            
            // Verifica si el usuario está registrado
            self.esta_registrado(caller)?;
            let usuario = self.usuarios.get(caller).unwrap();
            // Verifica si el usuario es un vendedor
            self.es_vendedor(usuario)?;

            let producto = Producto {
                id: self.siguiente_id_producto,
                nombre,
                descripcion,
                precio,
                cantidad,
                categoria,
                vendedor: caller, 
            };
            self.productos.push(producto);
            
            // Incrementa el ID del siguiente producto
            // saturating_add: si el valor se desborda, devuelve el valor máximo posible
            self.siguiente_id_producto = self.siguiente_id_producto.saturating_add(1);
            Ok(())
        }

        fn esta_registrado(&self, account: AccountId) -> Result<(), Error> {
            if self.usuarios.contains(account) {
                Ok(())
            } else {
                Err(Error::NoEstaRegistrado)
            }
        }

        fn es_vendedor(&self, account: Usuario) -> Result<(), Error> {
            if account.rol != Rol::Vendedor {
                return Err(Error::NoEsVendedor);    
            } 
            Ok(())
        }

        fn es_comprador(&self, account: Usuario) -> Result<(), Error> {
            if account.rol != Rol::Comprador {
                return Err(Error::NoEsComprador);    
            } 
            Ok(())
        }

        #[ink(message)]
        pub fn cambiar_rol(&mut self, nuevo_rol: Rol)-> Result<(), Error> {
            let caller = self.env().caller();

            // Verifica si el usuario está registrado
            self.esta_registrado(caller)?;
            
            // cambiar rol al caller
            let mut usuario = self.usuarios.get(caller).unwrap();
            usuario.rol = nuevo_rol;
            self.usuarios.insert(caller, &usuario);
            Ok(())
        }

        #[ink(message)]
        pub fn get_productos(&self) -> Result<Vec<Producto>, Error> {
            let caller = self.env().caller();
            
            // Verifica si el usuario está registrado
            self.esta_registrado(caller)?;
            
            // Obtiene el usuario del caller
            let usuario = self.usuarios.get(caller).unwrap();
            
            // Verifica si el usuario es un vendedor
            self.es_vendedor(usuario)?;

            // Itera sobre los productos y agrega los del vendedor
            let productos = self.productos_propios(caller)?;
            // for producto in self.productos.iter() {
            //     if producto.vendedor == caller {
            //         productos.push(producto.clone());
            //     }
            // }

            // // mostrar mensaje si no hay productos
            // if productos.is_empty() {
            //     return Err(Error::NoTieneProductos);
            // }
            Ok(productos)
        }

        fn productos_propios(&self, vendedor: AccountId) -> Result<Vec<Producto>, Error> {
            // productos.iter()
            //     .filter(|producto| producto.vendedor == vendedor)
            //     .cloned()
            //     .collect()
            self.productos.iter()
                .filter(|producto| producto.vendedor == vendedor)
                .cloned()
                .collect::<Vec<Producto>>()
                .try_into()
                .map_err(|_| Error::NoTieneProductos)
        }

        #[ink(message)]
        pub fn crear_orden(&mut self, id_producto: u32, cantidad: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            
            // Verifica si el usuario está registrado
            self.esta_registrado(caller)?;
            
            // Obtiene el usuario del caller
            let comprador = self.usuarios.get(caller).unwrap();
            
            // Verifica si el usuario es un comprador
            self.es_comprador(comprador.clone())?;

            // Verifica si el producto existe
            let producto_index = self.productos.iter()
                .position(|producto| producto.id == id_producto)
                .ok_or(Error::ProductoNoEncontrado)?;
            
            // Verifica si hay suficiente stock
            let producto = &mut self.productos[producto_index];
            if producto.cantidad < cantidad {
                return Err(Error::StockInsuficiente);
            }   

            // Reduce la cantidad del producto
            // saturating_sub: si el valor se vuelve negativo, devuelve 0
            // Rust no puede inferir que la verificacion anterior garantize que la cantidad no se vuelva negativa y provoque underflow
            // Por lo tanto, se usa saturating_sub para evitar underflow y el compilador no joda
            producto.cantidad = producto.cantidad.saturating_sub(cantidad); // Reduce la cantidad del producto

            // Obtiene el vendedor del producto
            let vendedor = self.usuarios.get(producto.vendedor)
                .ok_or(Error::NoEstaRegistrado)?;

            // Crea la orden
            let orden = Orden {
                comprador,
                vendedor, // Aquí deberías obtener el vendedor del producto
                producto_nombre: producto.nombre.clone(),
                cantidad,
                estado: EstadoOrden::Pendiente,
            };
            
            self.ordenes.push(orden);
            Ok(())
        }

        // fn cambiar_estado_orden(&mut self, orden: &mut Orden, nuevo_estado: EstadoOrden) {
        //     orden.estado = nuevo_estado;
        // }


    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// el mod tests es un módulo de pruebas que se ejecuta solo cuando se compila el código con la opción --test
        /// y se utiliza para probar el código del contrato inteligente.
        /// en este modulo, se definen pruebas unitarias para las funciones privadas que manejan la logica de negocio no de blockchain.
        /// Solo prueban algoritmos, cálculos, validaciones, etc.
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let market = Market::default();
            assert_eq!(market.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut market = Market::new(false);
            assert_eq!(market.get(), false);
            market.flip();
            assert_eq!(market.get(), true);
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {

        /// Este tipo de test 
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = MarketRef::default();

            // When
            let contract = client
                .instantiate("market", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Market>();

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = MarketRef::new(false);
            let contract = client
                .instantiate("market", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Market>();

            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = call_builder.flip();
            let _flip_result = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
