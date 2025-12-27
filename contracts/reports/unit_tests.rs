mod tests {
    use super::*;
    use market::{Orden, Producto, ReputacionUsuario};

    fn cuenta(n: u8) -> AccountId {
        AccountId::from([n; 32])
    }

    fn crear_producto(vendedor: u8, nombre: &str, categoria: &str, precio: u128) -> Producto {
        Producto {
            vendedor: cuenta(vendedor),
            nombre: String::from(nombre),
            descripcion: String::from("Descripción"),
            precio,
            stock: 10,
            categoria: String::from(categoria),
        }
    }

    fn crear_orden(comprador: u8, vendedor: u8, id_prod: u32, cantidad: u32, estado: Estado) -> Orden {
        Orden {
            comprador: cuenta(comprador),
            vendedor: cuenta(vendedor),
            id_prod,
            cantidad,
            estado,
            monto_total: 1000,
        }
    }

    fn crear_reputacion(como_vendedor: (u32, u32), como_comprador: (u32, u32)) -> ReputacionUsuario {
        ReputacionUsuario { como_vendedor, como_comprador }
    }

    fn crear_usuario_rep(id: u8, promedio: u32, cant: u32) -> UsuarioConReputacion {
        UsuarioConReputacion {
            usuario: cuenta(id),
            promedio_x100: promedio,
            cantidad_calificaciones: cant,
        }
    }

    fn crear_reputaciones_vendedores(data: &[(u8, u32, u32)]) -> Vec<(AccountId, ReputacionUsuario)> {
        data.iter()
            .map(|(id, suma, cant)| (cuenta(*id), crear_reputacion((*suma, *cant), (0, 0))))
            .collect()
    }

    fn crear_reputaciones_compradores(data: &[(u8, u32, u32)]) -> Vec<(AccountId, ReputacionUsuario)> {
        data.iter()
            .map(|(id, suma, cant)| (cuenta(*id), crear_reputacion((0, 0), (*suma, *cant))))
            .collect()
    }

    #[ink::test]
    fn test_constructor() {
        let addr1 = cuenta(10);
        let addr2 = cuenta(20);
        let reportes1 = Reportes::new(addr1);
        let reportes2 = Reportes::new(addr2);

        assert_eq!(reportes1.get_marketplace(), addr1);
        assert_eq!(reportes2.get_marketplace(), addr2);
        assert_ne!(reportes1.get_marketplace(), reportes2.get_marketplace());
    }

    #[ink::test]
    fn test_ordenar_por_reputacion() {
        let mut usuarios = vec![
            crear_usuario_rep(1, 100, 1),
            crear_usuario_rep(2, 500, 100),
            crear_usuario_rep(3, 350, 50),
            crear_usuario_rep(4, 500, 25),
            crear_usuario_rep(5, 200, 10),
        ];

        Reportes::_ordenar_por_reputacion(&mut usuarios);

        assert_eq!(usuarios[0].promedio_x100, 500);
        assert_eq!(usuarios[0].cantidad_calificaciones, 100);
        assert_eq!(usuarios[1].promedio_x100, 500);
        assert_eq!(usuarios[1].cantidad_calificaciones, 25);
        assert_eq!(usuarios[2].promedio_x100, 350);
        assert_eq!(usuarios[3].promedio_x100, 200);
        assert_eq!(usuarios[4].promedio_x100, 100);
        assert_eq!(usuarios.len(), 5);
    }

    #[ink::test]
    fn test_ordenar_por_reputacion_casos_borde() {
        let mut vacio: Vec<UsuarioConReputacion> = Vec::new();
        Reportes::_ordenar_por_reputacion(&mut vacio);
        assert!(vacio.is_empty());

        let mut uno = vec![crear_usuario_rep(1, 400, 5)];
        Reportes::_ordenar_por_reputacion(&mut uno);
        assert_eq!(uno.len(), 1);
        assert_eq!(uno[0].promedio_x100, 400);

        let mut empate = vec![crear_usuario_rep(1, 450, 10), crear_usuario_rep(2, 450, 10)];
        Reportes::_ordenar_por_reputacion(&mut empate);
        assert_eq!(empate.len(), 2);
        assert!(empate.iter().all(|u| u.promedio_x100 == 450));
    }

    #[ink::test]
    fn test_procesar_top_vendedores() {
        let reps = crear_reputaciones_vendedores(&[(1, 20, 5), (2, 25, 5), (3, 15, 5)]);
        let resultado = Reportes::_procesar_top_vendedores(reps, 5);
        assert_eq!(resultado.len(), 3);
        assert_eq!(resultado[0].usuario, cuenta(2));
        assert_eq!(resultado[0].promedio_x100, 500);

        let reps = crear_reputaciones_vendedores(&[(1, 20, 5), (2, 25, 5), (3, 15, 5), (4, 10, 5)]);
        let resultado = Reportes::_procesar_top_vendedores(reps, 2);
        assert_eq!(resultado.len(), 2);
        assert_eq!(resultado[0].promedio_x100, 500);

        let reps = vec![
            (cuenta(1), crear_reputacion((20, 4), (10, 2))),
            (cuenta(2), crear_reputacion((0, 0), (15, 3))),
            (cuenta(3), crear_reputacion((12, 3), (5, 1))),
        ];
        let resultado = Reportes::_procesar_top_vendedores(reps, 5);
        assert_eq!(resultado.len(), 2);
        assert_eq!(resultado[0].usuario, cuenta(1));
    }

    #[ink::test]
    fn test_procesar_top_vendedores_casos_borde() {
        let vacio: Vec<(AccountId, ReputacionUsuario)> = Vec::new();
        assert!(Reportes::_procesar_top_vendedores(vacio, 5).is_empty());

        let reps = crear_reputaciones_compradores(&[(1, 10, 2), (2, 15, 3)]);
        assert!(Reportes::_procesar_top_vendedores(reps, 5).is_empty());

        let reps = crear_reputaciones_vendedores(&[(1, 20, 5), (2, 15, 3)]);
        assert!(Reportes::_procesar_top_vendedores(reps, 0).is_empty());
    }

    #[ink::test]
    fn test_procesar_top_compradores() {
        let reps = crear_reputaciones_compradores(&[(1, 20, 5), (2, 25, 5), (3, 15, 5)]);
        let resultado = Reportes::_procesar_top_compradores(reps, 5);
        assert_eq!(resultado.len(), 3);
        assert_eq!(resultado[0].usuario, cuenta(2));
        assert_eq!(resultado[0].promedio_x100, 500);

        let reps = crear_reputaciones_compradores(&[(1, 20, 5), (2, 25, 5), (3, 15, 5)]);
        let resultado = Reportes::_procesar_top_compradores(reps, 1);
        assert_eq!(resultado.len(), 1);
        assert_eq!(resultado[0].promedio_x100, 500);
    }

    #[ink::test]
    fn test_procesar_top_compradores_casos_borde() {
        let reps = crear_reputaciones_vendedores(&[(1, 10, 2), (2, 15, 3)]);
        assert!(Reportes::_procesar_top_compradores(reps, 5).is_empty());

        let reps = crear_reputaciones_compradores(&[(1, 20, 5), (2, 15, 3)]);
        assert!(Reportes::_procesar_top_compradores(reps, 0).is_empty());
    }

    #[ink::test]
    fn test_procesar_productos_mas_vendidos() {
        let productos = vec![
            (1, crear_producto(1, "Laptop", "Electrónica", 1000)),
            (2, crear_producto(2, "Mouse", "Electrónica", 50)),
        ];
        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 2, Estado::Recibido)),
            (2, crear_orden(11, 1, 1, 3, Estado::Recibido)),
            (3, crear_orden(12, 2, 2, 1, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_productos_mas_vendidos(ordenes, productos, 5);
        assert_eq!(resultado.len(), 2);
        assert_eq!(resultado[0].id_producto, 1);
        assert_eq!(resultado[0].unidades_vendidas, 5);
        assert_eq!(resultado[1].unidades_vendidas, 1);
    }

    #[ink::test]
    fn test_procesar_productos_mas_vendidos_filtra_estados() {
        let productos = vec![(1, crear_producto(1, "Laptop", "Electrónica", 1000))];
        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 5, Estado::Recibido)),
            (2, crear_orden(11, 1, 1, 3, Estado::Pendiente)),
            (3, crear_orden(12, 1, 1, 2, Estado::Enviado)),
            (4, crear_orden(13, 1, 1, 1, Estado::Cancelada)),
        ];

        let resultado = Reportes::_procesar_productos_mas_vendidos(ordenes, productos, 5);
        assert_eq!(resultado.len(), 1);
        assert_eq!(resultado[0].unidades_vendidas, 5);
    }

    #[ink::test]
    fn test_procesar_productos_mas_vendidos_casos_borde() {
        let productos = vec![
            (1, crear_producto(1, "A", "Cat", 100)),
            (2, crear_producto(1, "B", "Cat", 100)),
            (3, crear_producto(1, "C", "Cat", 100)),
        ];
        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 10, Estado::Recibido)),
            (2, crear_orden(11, 1, 2, 5, Estado::Recibido)),
            (3, crear_orden(12, 1, 3, 3, Estado::Recibido)),
        ];
        let resultado = Reportes::_procesar_productos_mas_vendidos(ordenes, productos, 2);
        assert_eq!(resultado.len(), 2);

        let productos = vec![(1, crear_producto(1, "Laptop", "Electrónica", 1000))];
        assert!(Reportes::_procesar_productos_mas_vendidos(Vec::new(), productos, 5).is_empty());

        let productos = vec![(1u32, crear_producto(2, "Prod", "Cat", 100))];
        let ordenes = vec![(1u32, crear_orden(1, 2, 1, 5, Estado::Recibido))];
        assert!(Reportes::_procesar_productos_mas_vendidos(ordenes, productos, 0).is_empty());
    }

    #[ink::test]
    fn test_procesar_estadisticas_por_categoria() {
        let productos = vec![
            (1, crear_producto(1, "Laptop", "Electrónica", 1000)),
            (2, crear_producto(2, "Mouse", "Electrónica", 50)),
            (3, crear_producto(3, "Libro", "Libros", 20)),
        ];
        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 2, Estado::Recibido)),
            (2, crear_orden(11, 2, 2, 5, Estado::Recibido)),
            (3, crear_orden(12, 3, 3, 1, Estado::Recibido)),
        ];
        let calificaciones = vec![
            (String::from("Electrónica"), (45, 10)),
            (String::from("Libros"), (40, 10)),
        ];

        let resultado = Reportes::_procesar_estadisticas_por_categoria(productos, ordenes, calificaciones);
        assert_eq!(resultado.len(), 2);
        
        let electronica = resultado.iter().find(|s| s.categoria == "Electrónica").unwrap();
        assert_eq!(electronica.cantidad_productos, 2);
        assert_eq!(electronica.total_ventas, 2);
        assert_eq!(electronica.total_unidades, 7);
        assert_eq!(electronica.calificacion_promedio_x100, 450);

        let libros = resultado.iter().find(|s| s.categoria == "Libros").unwrap();
        assert_eq!(libros.cantidad_productos, 1);
        assert_eq!(libros.total_ventas, 1);
    }

    #[ink::test]
    fn test_procesar_estadisticas_por_categoria_casos_borde() {
        let productos = vec![(1, crear_producto(1, "Test", "NuevaCat", 100))];
        let ordenes = vec![(1, crear_orden(10, 1, 1, 3, Estado::Recibido))];
        let resultado = Reportes::_procesar_estadisticas_por_categoria(productos, ordenes, Vec::new());
        assert_eq!(resultado[0].calificacion_promedio_x100, 0);

        let productos = vec![(1, crear_producto(1, "Test", "Cat", 100))];
        let resultado = Reportes::_procesar_estadisticas_por_categoria(productos, Vec::new(), Vec::new());
        assert_eq!(resultado[0].total_ventas, 0);
        assert_eq!(resultado[0].cantidad_productos, 1);
    }

    #[ink::test]
    fn test_procesar_estadisticas_categoria() {
        let productos = vec![
            (1, crear_producto(1, "Laptop", "Electrónica", 1000)),
            (2, crear_producto(2, "Mouse", "Electrónica", 50)),
        ];
        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 2, Estado::Recibido)),
            (2, crear_orden(11, 2, 2, 3, Estado::Recibido)),
        ];

        let resultado = Reportes::_procesar_estadisticas_categoria(
            productos.clone(), ordenes.clone(), String::from("Electrónica"), (45, 10),
        );
        assert!(resultado.is_ok());
        let stats = resultado.unwrap();
        assert_eq!(stats.categoria, "Electrónica");
        assert_eq!(stats.cantidad_productos, 2);
        assert_eq!(stats.total_unidades, 5);
        assert_eq!(stats.calificacion_promedio_x100, 450);

        let resultado = Reportes::_procesar_estadisticas_categoria(
            productos.clone(), Vec::new(), String::from("NoExiste"), (0, 0),
        );
        assert_eq!(resultado.unwrap_err(), Error::CategoriaNoEncontrada);

        let productos = vec![(1, crear_producto(1, "Test", "Cat", 100))];
        let ordenes = vec![(1, crear_orden(10, 1, 1, 5, Estado::Recibido))];
        let resultado = Reportes::_procesar_estadisticas_categoria(productos, ordenes, String::from("Cat"), (0, 0));
        assert_eq!(resultado.unwrap().calificacion_promedio_x100, 0);

        let productos = vec![(1u32, crear_producto(1, "Prod", "Cat", 100))];
        let ordenes = vec![
            (1u32, crear_orden(2, 1, 1, 5, Estado::Cancelada)),
            (2u32, crear_orden(3, 1, 1, 3, Estado::Recibido)),
        ];
        let resultado = Reportes::_procesar_estadisticas_categoria(productos, ordenes, String::from("Cat"), (20, 5));
        let stats = resultado.unwrap();
        assert_eq!(stats.total_ventas, 1);
        assert_eq!(stats.total_unidades, 3);
    }

    #[ink::test]
    fn test_procesar_ordenes_por_usuario() {
        let ordenes = vec![
            (1, crear_orden(1, 10, 1, 2, Estado::Recibido)),
            (2, crear_orden(1, 11, 2, 3, Estado::Pendiente)),
            (3, crear_orden(1, 12, 3, 1, Estado::Recibido)),
        ];
        let resultado = Reportes::_procesar_ordenes_por_usuario(ordenes, cuenta(1));
        assert_eq!(resultado.ordenes_como_comprador, 3);
        assert_eq!(resultado.completadas_como_comprador, 2);
        assert_eq!(resultado.ordenes_como_vendedor, 0);

        let ordenes = vec![
            (1, crear_orden(10, 1, 1, 2, Estado::Recibido)),
            (2, crear_orden(11, 1, 2, 3, Estado::Enviado)),
            (3, crear_orden(12, 1, 3, 1, Estado::Recibido)),
        ];
        let resultado = Reportes::_procesar_ordenes_por_usuario(ordenes, cuenta(1));
        assert_eq!(resultado.ordenes_como_vendedor, 3);
        assert_eq!(resultado.completadas_como_vendedor, 2);
        assert_eq!(resultado.ordenes_como_comprador, 0);

        let ordenes = vec![
            (1, crear_orden(1, 10, 1, 2, Estado::Recibido)),
            (2, crear_orden(11, 1, 2, 3, Estado::Recibido)),
        ];
        let resultado = Reportes::_procesar_ordenes_por_usuario(ordenes, cuenta(1));
        assert_eq!(resultado.ordenes_como_comprador, 1);
        assert_eq!(resultado.ordenes_como_vendedor, 1);

        let ordenes = vec![(1, crear_orden(2, 3, 1, 2, Estado::Recibido))];
        let resultado = Reportes::_procesar_ordenes_por_usuario(ordenes, cuenta(1));
        assert_eq!(resultado.ordenes_como_comprador, 0);
        assert_eq!(resultado.ordenes_como_vendedor, 0);
    }

    #[ink::test]
    fn test_procesar_resumen_ordenes_todos_usuarios() {
        let usuarios = vec![cuenta(1), cuenta(2), cuenta(3)];
        let ordenes = vec![
            (1, crear_orden(1, 2, 1, 2, Estado::Recibido)),
            (2, crear_orden(1, 2, 2, 3, Estado::Pendiente)),
        ];

        let resultado = Reportes::_procesar_resumen_ordenes_todos_usuarios(usuarios, ordenes);
        assert_eq!(resultado.len(), 2);
        
        let u1 = resultado.iter().find(|u| u.usuario == cuenta(1)).unwrap();
        assert_eq!(u1.ordenes_como_comprador, 2);

        let u2 = resultado.iter().find(|u| u.usuario == cuenta(2)).unwrap();
        assert_eq!(u2.ordenes_como_vendedor, 2);

        assert!(!resultado.iter().any(|u| u.usuario == cuenta(3)));
    }

    #[ink::test]
    fn test_procesar_resumen_ordenes_todos_usuarios_vacio() {
        let usuarios = vec![cuenta(1), cuenta(2)];
        assert!(Reportes::_procesar_resumen_ordenes_todos_usuarios(usuarios, Vec::new()).is_empty());
    }

    #[ink::test]
    fn test_procesar_resumen_general() {
        let ordenes = vec![
            (1, crear_orden(1, 2, 1, 2, Estado::Recibido)),
            (2, crear_orden(1, 2, 2, 3, Estado::Pendiente)),
            (3, crear_orden(1, 2, 3, 1, Estado::Recibido)),
        ];
        let resultado = Reportes::_procesar_resumen_general(5, 10, ordenes);
        assert_eq!(resultado, (5, 10, 3, 2));

        assert_eq!(Reportes::_procesar_resumen_general(0, 0, Vec::new()), (0, 0, 0, 0));

        let ordenes = vec![
            (1, crear_orden(1, 2, 1, 2, Estado::Pendiente)),
            (2, crear_orden(1, 2, 2, 3, Estado::Enviado)),
            (3, crear_orden(1, 2, 3, 1, Estado::Cancelada)),
        ];
        let resultado = Reportes::_procesar_resumen_general(2, 3, ordenes);
        assert_eq!(resultado.2, 3);
        assert_eq!(resultado.3, 0);

        let ordenes = vec![
            (1u32, crear_orden(1, 2, 1, 5, Estado::Cancelada)),
            (2u32, crear_orden(1, 2, 1, 3, Estado::Recibido)),
            (3u32, crear_orden(1, 2, 1, 2, Estado::Pendiente)),
        ];
        let resultado = Reportes::_procesar_resumen_general(5, 10, ordenes);
        assert_eq!(resultado.3, 1);
    }

    #[ink::test]
    fn test_procesar_listar_categorias() {
        let productos = vec![
            (1, crear_producto(1, "A", "Electrónica", 100)),
            (2, crear_producto(2, "B", "Libros", 50)),
            (3, crear_producto(3, "C", "Electrónica", 200)),
        ];
        let resultado = Reportes::_procesar_listar_categorias(&productos);
        assert_eq!(resultado.len(), 2);
        assert!(resultado.contains(&String::from("Electrónica")));
        assert!(resultado.contains(&String::from("Libros")));

        let vacio: Vec<(u32, Producto)> = Vec::new();
        assert!(Reportes::_procesar_listar_categorias(&vacio).is_empty());

        let productos = vec![
            (1, crear_producto(1, "A", "Única", 100)),
            (2, crear_producto(2, "B", "Única", 200)),
        ];
        let resultado = Reportes::_procesar_listar_categorias(&productos);
        assert_eq!(resultado.len(), 1);
        assert_eq!(resultado[0], "Única");
    }

    #[ink::test]
    fn test_structs_clone_eq() {
        let usuario = crear_usuario_rep(1, 450, 10);
        assert_eq!(usuario.clone(), usuario);

        let producto = ProductoVendido {
            id_producto: 1,
            nombre: String::from("Laptop"),
            categoria: String::from("Electrónica"),
            vendedor: cuenta(5),
            unidades_vendidas: 100,
        };
        assert_eq!(producto.clone(), producto);

        let stats = EstadisticasCategoria {
            categoria: String::from("Electrónica"),
            total_ventas: 150,
            total_unidades: 500,
            calificacion_promedio_x100: 425,
            cantidad_productos: 25,
        };
        assert_eq!(stats.clone(), stats);

        let ordenes = OrdenesUsuario {
            usuario: cuenta(10),
            ordenes_como_comprador: 15,
            ordenes_como_vendedor: 25,
            completadas_como_comprador: 12,
            completadas_como_vendedor: 20,
        };
        assert_eq!(ordenes.clone(), ordenes);

        let error = Error::CategoriaNoEncontrada;
        assert_eq!(error, Error::CategoriaNoEncontrada);
        let _ = format!("{:?}", error);
    }
}