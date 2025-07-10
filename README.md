# Agora Market

Este repositorio contiene los contratos inteligentes del proyecto Agora Market.

## Estructura del Proyecto

agoramarket/market
|
|__.gitignore
|__LICENSE
|__README.md
|__contracts
    |
    |__market
        |__Cargo.toml
        |__Cargo.lock
        |__lib.rs

## Instalación

1. Clona este repositorio.
2. Entra a la carpeta `contracts/market`.
3. Instala dependencias y compila el contrato usando Cargo:
   ```bash
   cargo install cargo-contract
   cargo contract build
   ```

## Descripción General

El contrato de Agora Market implementa la lógica principal de un marketplace descentralizado utilizando Rust.

## Licencia

Este proyecto se distribuye bajo la licencia que encontrarás en el archivo LICENSE.
