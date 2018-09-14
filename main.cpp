#include <iostream>
#include <windows.h>
#include <wrl/client.h>
#include "CLI11.hpp"
#include "dxcapi.h"

using Microsoft::WRL::ComPtr;

int main(int argc, const char* argv[])
{
	CLI::App app{ "DXIL Signing Utility" };

	std::string input_file = "";
	app.add_option("-i,--input", input_file, "Input unsigned dxil file")->required();

	std::string output_file = "";
	app.add_option("-o,--output", output_file, "Output signed dxil file")->required();

	CLI11_PARSE(app, argc, argv);

	std::cout << "Loading input file: " << input_file << std::endl;

	FILE* input_fh = fopen(input_file.c_str(), "rb");
	if (input_fh == nullptr)
	{
		std::cout << "Failed to open input file" << std::endl;
		exit(1);
	}

	fseek(input_fh, 0, SEEK_END);
	size_t input_size = ftell(input_fh);
	fseek(input_fh, 0, SEEK_SET);

	std::vector<uint8_t> input_data;
	input_data.resize(input_size);
	size_t bytes_read = fread(input_data.data(), 1, input_data.size(), input_fh);
	fclose(input_fh);

	if (bytes_read != input_data.size())
	{
		std::cout << "Failed to read input file" << std::endl;
		exit(1);
	}

	HMODULE dxil_module = ::LoadLibrary("dxil.dll");
	if (dxil_module == nullptr)
	{
		std::cout << "Failed to load dxil.dll" << std::endl;
		exit(1);
	}

	DxcCreateInstanceProc dxil_create_func = (DxcCreateInstanceProc)GetProcAddress(dxil_module, "DxcCreateInstance");
	if (dxil_create_func == nullptr)
	{
		std::cout << "Failed to get dxil create proc" << std::endl;
		exit(1);
	}

	HMODULE dxc_module = ::LoadLibrary("dxcompiler.dll");
	if (dxc_module == nullptr)
	{
		std::cout << "Failed to load dxcompiler.dll" << std::endl;
		exit(1);
	}

	DxcCreateInstanceProc dxc_create_func = (DxcCreateInstanceProc)GetProcAddress(dxc_module, "DxcCreateInstance");
	if (dxc_create_func == nullptr)
	{
		std::cout << "Failed to get dxc create proc" << std::endl;
		exit(1);
	}

	ComPtr<IDxcLibrary> library;
	dxc_create_func(CLSID_DxcLibrary, __uuidof(IDxcLibrary), (void**)&library);

	ComPtr<IDxcBlobEncoding> containerBlob;
	library->CreateBlobWithEncodingFromPinned((BYTE*)input_data.data(), (UINT32)input_data.size(), 0 /* binary, no code page */, containerBlob.GetAddressOf());

	ComPtr<IDxcValidator> validator;
	if (FAILED(dxc_create_func(CLSID_DxcValidator, __uuidof(IDxcValidator), (void**)&validator)))
	{
		std::cout << "Failed to create validator instance" << std::endl;
		exit(1);
	}

	ComPtr<IDxcOperationResult> result;
	if (FAILED(validator->Validate(containerBlob.Get(), DxcValidatorFlags_InPlaceEdit, &result)))
	{
		std::cout << "Failed to validate dxil container" << std::endl;
		exit(1);
	}

	HRESULT validateStatus;
	if (FAILED(result->GetStatus(&validateStatus)))
	{
		std::cout << "Failed to get dxil validate status" << std::endl;
		exit(1);
	}

	if (FAILED(validateStatus))
	{
		std::cout << "The dxil container failed validation" << std::endl;

		std::string errorString;

		ComPtr<IDxcBlobEncoding> printBlob, printBlobUtf8;
		result->GetErrorBuffer(&printBlob);

		library->GetBlobAsUtf8(printBlob.Get(), printBlobUtf8.GetAddressOf());
		if (printBlobUtf8)
		{
			errorString = reinterpret_cast<const char*>(printBlobUtf8->GetBufferPointer());
		}

		std::cout << "Error: " << std::endl << errorString << std::endl;

		exit(2);
	}

	ComPtr<IDxcBlob> validatedBlob;
	if (FAILED(result->GetResult(&validatedBlob)))
	{
		std::cout << "Failed to get validated dxil blob" << std::endl;
		exit(1);
	}

	validator = nullptr;

	std::cout << "Saving output file: " << output_file << std::endl;

	FILE* output_fh = fopen(output_file.c_str(), "wb");
	if (output_fh == nullptr)
	{
		std::cout << "Failed to create output file" << std::endl;
		exit(1);
	}

	size_t bytes_written = fwrite(validatedBlob->GetBufferPointer(), 1, validatedBlob->GetBufferSize(), output_fh);
	fclose(output_fh);

	if (bytes_written != validatedBlob->GetBufferSize())
	{
		std::cout << "Failed to write output file" << std::endl;
		exit(1);
	}

	::FreeLibrary(dxc_module);
	::FreeLibrary(dxil_module);
}