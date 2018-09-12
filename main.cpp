#include <iostream>
#include "CLI11.hpp"

int main(int argc, const char* argv[])
{
    CLI::App app{"DXIL Signing Utility"};

    std::string input_file = "";
    app.add_option("-i,--input", input_file, "Input unsigned dxil file")->required();

    std::string output_file = "";
    app.add_option("-o,--output", output_file, "Output signed dxil file")->required();

    CLI11_PARSE(app, argc, argv);

    std::cout << "Loading input file: " << input_file << std::endl;

    std::cout << "Saving output file: " << output_file << std::endl;
}